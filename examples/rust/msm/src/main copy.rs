use icicle_bn254::curve::{CurveCfg, G1Projective, G2CurveCfg, G2Projective, ScalarCfg};

use icicle_cuda_runtime::{
    memory::{DeviceVec, HostSlice},
    stream::CudaStream,
};

use icicle_core::{curve::Curve, msm, traits::GenerateRandom};

#[cfg(feature = "arkworks")]
use icicle_core::traits::ArkConvertible;

#[cfg(feature = "arkworks")]
use ark_bls12_377::{Fr as Bls12377Fr, G1Affine as Bls12377G1Affine, G1Projective as Bls12377ArkG1Projective};
#[cfg(feature = "arkworks")]
use ark_bn254::{Fr as Bn254Fr, G1Affine as Bn254G1Affine, G1Projective as Bn254ArkG1Projective};
#[cfg(feature = "arkworks")]
use ark_ec::scalar_mul::variable_base::VariableBaseMSM;

use lambdaworks_math::{
    cyclic_group::IsGroup,
    elliptic_curve::traits::IsPairing,
    elliptic_curve::{
        short_weierstrass::{
            curves::bls12_381::{curve::BLS12381Curve, default_types::FrElement},
            point::ShortWeierstrassProjectivePoint,
        },
        traits::IsEllipticCurve,
    },
    errors::DeserializationError,
    field::{element::FieldElement, traits::IsPrimeField},
    msm::pippenger,
    traits::{AsBytes, Deserializable},
    unsigned_integer::element::UnsignedInteger,
};

// #[cfg(feature = "lamdaworks")]
use std::str::FromStr;
// #[cfg(feature = "lamdaworks")]
type G1Point = <BLS12381Curve as IsEllipticCurve>::PointRepresentation;

#[cfg(feature = "profile")]
use std::time::Instant;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Lower bound (inclusive) of MSM sizes to run for
    #[arg(short, long, default_value_t = 19)]
    lower_bound_log_size: u8,

    /// Upper bound of MSM sizes to run for
    #[arg(short, long, default_value_t = 22)]
    upper_bound_log_size: u8,
}

fn main() {
    let args = Args::parse();
    let lower_bound = args.lower_bound_log_size;
    let upper_bound = args.upper_bound_log_size;
    println!("Running Icicle Examples: Rust MSM");
    let upper_size = 1 << (upper_bound);
    println!("Generating random inputs on host for bn254...");
    let upper_points = CurveCfg::generate_random_affine_points(upper_size);
    let g2_upper_points = G2CurveCfg::generate_random_affine_points(upper_size);
    let upper_scalars = ScalarCfg::generate_random(upper_size);

    // println!("lower_bound: {:?}", lower_bound);
    // println!("upper_bound: {:?}", upper_bound);

    println!("upper_points: {:?}", &upper_points[..10]);
    // println!("g2_upper_points: {:?}", g2_upper_points);
    println!("upper_scalars: {:?}", &upper_scalars[..10]);

    for i in lower_bound..=upper_bound {
        let log_size = i;
        let size = 1 << log_size;
        println!(
            "---------------------- MSM size 2^{}={} ------------------------",
            log_size, size
        );
        // Setting Bn254 points and scalars
        let points = HostSlice::from_slice(&upper_points[..size]);
        let g2_points = HostSlice::from_slice(&g2_upper_points[..size]);
        let scalars = HostSlice::from_slice(&upper_scalars[..size]);
        // let scalars_sample = HostSlice::from_slice(&upper_scalars[..10]);
        // let points_sample = HostSlice::from_slice(&upper_points[..10]);

        // println!("points: {:?}", points_sample);
        // println!("g2_points: {:?}", g2_points[0]);
        // println!("scalars: {:?}", scalars_sample);
        
        println!("Configuring bn254 MSM...");
        let mut msm_results = DeviceVec::<G1Projective>::cuda_malloc(1).unwrap();
        // let mut msm_results_sample = DeviceVec::<G1Projective>::cuda_malloc(1).unwrap();
        let mut g2_msm_results = DeviceVec::<G2Projective>::cuda_malloc(1).unwrap();
        let stream = CudaStream::create().unwrap();
        let g2_stream = CudaStream::create().unwrap();
        let mut cfg = msm::MSMConfig::default();
        let mut g2_cfg = msm::MSMConfig::default();
        cfg.ctx
            .stream = &stream;
        g2_cfg
            .ctx
            .stream = &g2_stream;
        cfg.is_async = true;
        g2_cfg.is_async = true;

        // msm::msm(scalars_sample, points_sample, &cfg, &mut msm_results_sample[..]).unwrap();

        println!("Executing bn254 MSM on device...");
        #[cfg(feature = "profile")]
        let start = Instant::now();
        msm::msm(scalars, points, &cfg, &mut msm_results[..]).unwrap();
        #[cfg(feature = "profile")]
        println!(
            "ICICLE BN254 MSM on size 2^{log_size} took: {} ms",
            start
                .elapsed()
                .as_millis()
        );
        msm::msm(scalars, g2_points, &g2_cfg, &mut g2_msm_results[..]).unwrap();

        println!("Moving results to host..");
        let mut msm_host_result = vec![G1Projective::zero(); 1];
        let mut g2_msm_host_result = vec![G2Projective::zero(); 1];

        stream
            .synchronize()
            .unwrap();
        g2_stream
            .synchronize()
            .unwrap();
        msm_results
            .copy_to_host(HostSlice::from_mut_slice(&mut msm_host_result[..]))
            .unwrap();
        g2_msm_results
            .copy_to_host(HostSlice::from_mut_slice(&mut g2_msm_host_result[..]))
            .unwrap();
        println!("bn254 result: {:#?}", msm_host_result);
        println!("G2 bn254 result: {:#?}", g2_msm_host_result);

        #[cfg(feature = "lambdaworks")]
        {
            println!("Checking against lambdaworks...");
            let lamda_points: Vec<G1Point> = upper_points
                .iter()
                .map(|p| G1Point::new([
                    FieldElement::from_hex(&p.x.to_string()).unwrap(),
                    FieldElement::from_hex(&p.y.to_string()).unwrap(),
                    FieldElement::one(),
                ]))
                .collect();

            let lamda_scalars: Vec<UnsignedInteger<4>> = upper_scalars
                .iter()
                .map(|s| UnsignedInteger::<4>::from_hex(&s.to_string()).unwrap())
                .collect();

            #[cfg(feature = "profile")]
            let start = Instant::now();
            let msm_result = pippenger::msm(&lamda_scalars, &lamda_points).expect("MSM calculation failed");
            println!("Lambdaworks result: {:?}", msm_result);

            #[cfg(feature = "profile")]
            println!(
                "Lambda bls12_381 MSM on size 2^{log_size} took: {} ms",
                start
                    .elapsed()
                    .as_millis()
            );
        }

        #[cfg(feature = "arkworks")]
        {
            println!("Checking against arkworks...");
            let ark_points: Vec<Bn254G1Affine> = points
                .iter()
                .map(|&point| point.to_ark())
                .collect();
            let ark_scalars: Vec<Bn254Fr> = scalars
                .iter()
                .map(|scalar| scalar.to_ark())
                .collect();

            #[cfg(feature = "profile")]
            let start = Instant::now();
            let bn254_ark_msm_res = Bn254ArkG1Projective::msm(&ark_points, &ark_scalars).unwrap();
            println!("Arkworks Bn254 result: {:#?}", bn254_ark_msm_res);
            #[cfg(feature = "profile")]
            println!(
                "Ark BN254 MSM on size 2^{log_size} took: {} ms",
                start
                    .elapsed()
                    .as_millis()
            );

            let bn254_icicle_msm_res_as_ark = msm_host_result[0].to_ark();

            println!(
                "Bn254 MSM is correct: {}",
                bn254_ark_msm_res.eq(&bn254_icicle_msm_res_as_ark)
            );
        
        }

        println!("Cleaning up bn254...");
        stream
            .destroy()
            .unwrap();
        
        println!("");
    }
}


