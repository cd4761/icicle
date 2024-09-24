use icicle_bls12_381::curve::{CurveCfg, G1Projective, ScalarCfg};

use icicle_cuda_runtime::{
    memory::{DeviceVec, HostSlice},
    stream::CudaStream,
};

use icicle_core::{curve::{Curve, Affine}, msm, traits::GenerateRandom};

#[cfg(feature = "arkworks")]
use icicle_core::traits::ArkConvertible;

#[cfg(feature = "arkworks")]
use ark_bls12_377::{Fr as Bls12377Fr, G1Affine as Bls12377G1Affine, G1Projective as Bls12377ArkG1Projective};
#[cfg(feature = "arkworks")]
use ark_bn254::{Fr as Bn254Fr, G1Affine as Bn254G1Affine, G1Projective as Bn254ArkG1Projective};
#[cfg(feature = "arkworks")]
use ark_ec::scalar_mul::variable_base::VariableBaseMSM;

use lambdaworks_math::{
    //cyclic_group::IsGroup,
    elliptic_curve::traits::{
        //IsPairing, 
        FromAffine
    },
    elliptic_curve::{
        short_weierstrass::{
            curves::bls12_381::{
                curve::BLS12381Curve, 
                field_extension::BLS12381PrimeField
            },
            point::{ShortWeierstrassProjectivePoint, PointFormat, Endianness},
        },
        traits::IsEllipticCurve,
    },
    //errors::DeserializationError,
    field::element::FieldElement,
    msm::pippenger,
    unsigned_integer::element::UnsignedInteger,
};

// #[cfg(feature = "lamdaworks")]
type G1Point = <BLS12381Curve as IsEllipticCurve>::PointRepresentation;
type BLS_G1Point = ShortWeierstrassProjectivePoint<BLS12381Curve>;

#[cfg(feature = "profile")]
use std::time::Instant;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Lower bound (inclusive) of MSM sizes to run for
    #[arg(short, long, default_value_t = 19)]
    lower_bound_log_size: u8,

    /// Upper bound of MSM sizes to run for
    #[arg(short, long, default_value_t = 19)]
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
    // let g2_upper_points = G2CurveCfg::generate_random_affine_points(upper_size);
    let upper_scalars = ScalarCfg::generate_random(upper_size);

    for i in lower_bound..=upper_bound {
        let log_size = i;
        let size = 1 << log_size;
        println!(
            "---------------------- MSM size 2^{}={} ------------------------",
            log_size, size
        );
        // Setting Bn254 points and scalars
        let points = HostSlice::from_slice(&upper_points[..size]);
        // let g2_points = HostSlice::from_slice(&g2_upper_points[..size]);
        let scalars = HostSlice::from_slice(&upper_scalars[..size]);

        println!("points: {:?}", points[0]);
        // println!("g2_points: {:?}", g2_points[0]);
        // println!("scalars: {:?}", scalars_sample);
        
        println!("Configuring bls12-381 MSM...");
        let mut msm_results = DeviceVec::<G1Projective>::cuda_malloc(1).unwrap();

        let stream = CudaStream::create().unwrap();
        let mut cfg = msm::MSMConfig::default();
    
        cfg.ctx
            .stream = &stream;
     
        cfg.is_async = true;
   
        println!("Executing bls12-381 MSM on device...");
        #[cfg(feature = "profile")]
        let start = Instant::now();
        msm::msm(scalars, points, &cfg, &mut msm_results[..]).unwrap();
        #[cfg(feature = "profile")]
        println!(
            "ICICLE bls12-381 MSM on size 2^{log_size} took: {} ms",
            start
                .elapsed()
                .as_millis()
        );

        println!("Moving results to host..");
        let mut msm_host_result = vec![G1Projective::zero(); 1];

        stream
            .synchronize()
            .unwrap();
      
        msm_results
            .copy_to_host(HostSlice::from_mut_slice(&mut msm_host_result[..]))
            .unwrap();
      
        println!("bls12-381 result: {:#?}", msm_host_result);
      
        // println!("G2 bls12-381 result: {:#?}", g2_msm_host_result);

        #[cfg(feature = "lambdaworks")]
        {
            println!("Checking against lambdaworks...");
           
            let lamda_points: Vec<BLS_G1Point> = upper_points[..size]
                 .iter()
                 .map(|p| {
                    BLS_G1Point::from_affine(
                        FieldElement::from_hex(&p.x.to_string()).unwrap(),
                        FieldElement::from_hex(&p.y.to_string()).unwrap(),
                    )
                    .unwrap() 
         
                 })
                 .collect();  

            // let lambda_point_x = &lamda_points[0].to_affine().x().value().limbs
            //     .iter()
            //     .map(|limb| format!("{:016x}", limb)).collect::<String>();
            let affine_lambda = lamda_points[0].to_affine();
            let [x, y, z] = affine_lambda.coordinates();

            println!("Icicle points x: {:?}", to_limbs(points[0].x.to_string()));
            println!("Lambdaworks points x: {:?}", x);
          

            let lamda_scalars: Vec<UnsignedInteger<6>> = upper_scalars[..size]
                .iter()
                .map(|s| UnsignedInteger::<6>::from_hex(&s.to_string()).unwrap())
                .collect();

            println!("");
    

            // scalar 비교
            let lambda_scalar = lamda_scalars[0].limbs
                .iter()
                .map(|limb| format!("{:016x}", limb)).collect::<String>();

            println!("Icicle scalar: {:?}", UnsignedInteger::<6>::from_hex(&scalars[0].to_string()).unwrap());
            println!("Lambdaworks scalar: {:?}", lamda_scalars[0]);

            #[cfg(feature = "profile")]
            let start = Instant::now();
            let lambda_result = pippenger::msm(&lamda_scalars, &lamda_points).expect("MSM calculation failed");

            let affine_result: Affine<CurveCfg> = msm_host_result[0].into();
            println!("host affine_result: {:?}", affine_result);
            println!("host affine_result x: {:?}", to_limbs(affine_result.x.to_string()));
            println!("host affine_result y: {:?}", to_limbs(affine_result.y.to_string()));
         
            let lambda_affine_x_convert: String = lambda_result.to_affine().x()
                .value().limbs.iter().map(|limb| format!("{:016x}", limb)).collect::<String>();
            let lambda_affine_y_convert: String = lambda_result.to_affine().y()
                .value().limbs.iter().map(|limb| format!("{:016x}", limb)).collect::<String>();
            
            // println!("lambda affine_result: {:?}", lambda_result.to_affine());
            println!("lambda affine_result x: {:?}", lambda_result.to_affine().x().value());
            println!("lambda affine_result y: {:?}", lambda_result.to_affine().y().value());

            let hex_affine_result: Vec<String> = to_icicle(&lambda_result.to_affine());
            let hex_affine_lambda: Vec<UnsignedInteger<6>> = hex_affine_result
                .iter()
                .map(|hex| 
                    UnsignedInteger::<6>::from_hex(&hex.to_string()).unwrap()
                )
                .collect();

            let hex_result: Vec<String> = to_icicle(&lambda_result);  
            println!("Checking with tolerance...");

            // let tolerance = EPSILON * 10.0;
            
            // if are_values_equal(&affine_result.x, &lambda_affine_x_convert, tolerance) {
            //     println!("X coordinates are equal within tolerance.");
            // } else {
            //     println!("X coordinates differ!");
            // }
                
            //println!("hex_result: {:?}", hex_result);
            
            //println!("hex_affine_result: {:?}", hex_affine_result);

            // println!("hex_lambda: {:?}", hex_affin/e/_lambda);
            // println!("&msm_result.to_affine(): {:?} ", &msm_result.to_affine());

            #[cfg(feature = "profile")]
            println!(
                "Lambda bls12_381 MSM on size 2^{log_size} took: {} ms",
                start
                    .elapsed()
                    .as_millis()
            );
        }

        println!("Cleaning up bls12-381...");
        stream
            .destroy()
            .unwrap();
        
        println!("");
    }
}

fn to_icicle(point: &BLS_G1Point) -> Vec<String> {
    point.0.value.iter().map(|ui| {
        let value = ui.value();
        value.limbs.iter().map(|limb| format!("{:016x}", limb)).collect::<String>()
    }).collect()
}

fn to_limbs(hex: String) -> UnsignedInteger::<6> {
    let limb = UnsignedInteger::<6>::from_hex(&hex).unwrap();
    limb
}
