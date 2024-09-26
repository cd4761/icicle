use icicle_bls12_381::curve;
use icicle_cuda_runtime::{
    memory::{DeviceVec, HostSlice},
    stream::CudaStream,
};

use icicle_core::{curve::{Curve, Affine}, msm, traits::{GenerateRandom, FieldImpl}};
use lambdaworks_math::{
    elliptic_curve::{
        short_weierstrass::{
            curves::bls12_381::{curve::{BLS12381Curve, BLS12381FieldElement}},
            point::ShortWeierstrassProjectivePoint,
        },
        traits::IsEllipticCurve,
    },
    field::element::FieldElement,
    unsigned_integer::element::UnsignedInteger,
    msm::pippenger,
    traits::ByteConversion,
    errors::ByteConversionError,
};
use hex;

type G1Point = <BLS12381Curve as IsEllipticCurve>::PointRepresentation;
type BLS_G1Point = ShortWeierstrassProjectivePoint<BLS12381Curve>;
type BlsG1point = ShortWeierstrassProjectivePoint<BLS12381Curve>;

#[cfg(feature = "profile")]
use std::time::Instant;

use clap::Parser;

trait PointConversion {
    fn to_icicle(&self) -> curve::G1Affine;
    fn from_icicle(icicle: &curve::G1Projective) -> Result<Self, ByteConversionError>
    where
        Self: Sized;
}

impl PointConversion for BlsG1point {
    fn to_icicle(&self) -> curve::G1Affine {
        let s = self.to_affine();
        let x = s.x().to_icicle();
        let y = s.y().to_icicle();
        curve::G1Affine { x, y }
    }

    fn from_icicle(icicle: &curve::G1Projective) -> Result<Self, ByteConversionError> {
        Ok(Self::new([
            ToIcicle::from_icicle(&icicle.x)?,
            ToIcicle::from_icicle(&icicle.y)?,
            ToIcicle::from_icicle(&icicle.z)?,
        ]))
    }
}

trait ToIcicle {
    fn to_icicle_scalar(&self) -> curve::ScalarField;
    fn to_icicle(&self) -> curve::BaseField;
    fn from_icicle(icicle: &curve::BaseField) -> Result<Self, ByteConversionError>
    where
        Self: Sized;
}

impl ToIcicle for BLS12381FieldElement {
    fn to_icicle_scalar(&self) -> curve::ScalarField {
        let scalar_bytes = self.to_bytes_le();
        curve::ScalarField::from_bytes_le(&scalar_bytes) 

    fn to_icicle(&self) -> curve::BaseField {
        curve::BaseField::from_bytes_le(&self.to_bytes_le()) 
    }

    fn from_icicle(icicle: &curve::BaseField) -> Result<Self, ByteConversionError> {
        Self::from_bytes_le(&icicle.to_bytes_le()) 
    }
}

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
    let upper_points = curve::CurveCfg::generate_random_affine_points(upper_size);
    let upper_scalars = curve::ScalarCfg::generate_random(upper_size);

    for i in lower_bound..=upper_bound {
        let log_size = i;
        let size = 1 << log_size;
        println!(
            "---------------------- MSM size 2^{}={} ------------------------",
            log_size, size
        );
        let points = HostSlice::from_slice(&upper_points[..size]);
        let scalars = HostSlice::from_slice(&upper_scalars[..size]);

        println!("points: {:?}", points[0]);

        println!("Configuring bls12-381 MSM...");
        let mut msm_results = DeviceVec::<curve::G1Projective>::cuda_malloc(1).unwrap();
        let stream = CudaStream::create().unwrap();
        let mut cfg = msm::MSMConfig::default();
        cfg.ctx.stream = &stream;
        cfg.is_async = true;

        println!("Executing bls12-381 MSM on device...");
        msm::msm(scalars, points, &cfg, &mut msm_results[..]).unwrap();

        println!("Moving results to host..");
        let mut msm_host_result = vec![curve::G1Projective::zero(); 1];
        stream.synchronize().unwrap();
        msm_results.copy_to_host(HostSlice::from_mut_slice(&mut msm_host_result[..])).unwrap();
        println!("bls12-381 result: {:#?}", msm_host_result);
        
        let res = <ShortWeierstrassProjectivePoint<BLS12381Curve> as PointConversion>::from_icicle(&msm_host_result[0]).unwrap();
        println!("Converted result: {:?}", res);
        #[cfg(feature = "lambdaworks")]
        {
            println!("Checking against lambdaworks...");
           
            let lamda_points: Vec<G1Point> = upper_points[..size]
                 .iter()
                 .map(|p| {
                    G1Point::new([
                        FieldElement::from_hex(&p.x.to_string()).unwrap(),
                        FieldElement::from_hex(&p.y.to_string()).unwrap(),
                        FieldElement::one()
                    ])
                 })
                 .collect();  

            let lamda_scalars: Vec<UnsignedInteger<6>> = upper_scalars[..size]
                .iter()
                .map(|s| UnsignedInteger::<6>::from_hex(&s.to_string()).unwrap())
                .collect();

            println!("");

            // scalar 비교
            let lambda_scalar = lamda_scalars[0].limbs
                .iter()
                .map(|limb| format!("{:016x}", limb)).collect::<String>();

            #[cfg(feature = "profile")]
            let start = Instant::now();
            let lambda_result = pippenger::msm(&lamda_scalars, &lamda_points).expect("MSM calculation failed");

            assert_eq!(res, lambda_result);
            println!("Checking with tolerance...");

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
