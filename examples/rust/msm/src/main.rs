use icicle_bls12_381::curve;
use icicle_core::{error::IcicleError, msm, traits::FieldImpl};
use icicle_cuda_runtime::{ memory::{DeviceVec, HostSlice}, stream::CudaStream};
use lambdaworks_math::{
    elliptic_curve::short_weierstrass::{
        curves::bls12_381::{curve::{BLS12381Curve, BLS12381FieldElement}},
        point::ShortWeierstrassProjectivePoint,
    },
    field::element::FieldElement,
    unsigned_integer::element::UnsignedInteger,
    traits::ByteConversion,
    errors::ByteConversionError,
    msm::pippenger,
};
use hex;

// Type alias for better readability
type BlsG1point = ShortWeierstrassProjectivePoint<BLS12381Curve>;

// Trait to handle scalar conversion
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
    }

    fn to_icicle(&self) -> curve::BaseField {
        curve::BaseField::from_bytes_le(&self.to_bytes_le())
    }

    fn from_icicle(icicle: &curve::BaseField) -> Result<Self, ByteConversionError> {
        Self::from_bytes_le(&icicle.to_bytes_le())
    }
}

// Trait for point conversion to icicle format
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

fn main() {
    const LEN: usize = 20;
    let eight: BLS12381FieldElement = FieldElement::from(8);

    // Convert to UnsignedInteger as expected by msm
    let scalars = vec![eight; LEN];
    let lambda_scalars: Vec<UnsignedInteger<6>> = scalars
        .iter()
        .map(|scalar| {
            scalar.representative()
            // let bytes = scalar.to_bytes_le();
            // UnsignedInteger::from_hex(&hex::encode(bytes)).expect("Conversion failed")
        })
        .collect();
    
    
    let lambda_points = (0..LEN).map(|_| point_times_5()).collect::<Vec<_>>();

    let expected = pippenger::msm(
        &lambda_scalars,
        &lambda_points,
    )
    .unwrap();

    let res = bls12_381_g1_msm(&scalars, &lambda_points, None);
        
    assert_eq!(res, Ok(expected));
}

pub fn bls12_381_g1_msm(
    scalars:&[BLS12381FieldElement],
    points: &[BlsG1point],
    config: Option<msm::MSMConfig>,
) -> Result<BlsG1point, IcicleError> {
    let mut cfg = config.unwrap_or(msm::MSMConfig::default());

    let convert_scalars = scalars.iter()
            .map(|scalar| ToIcicle::to_icicle_scalar(scalar))
            .collect::<Vec<_>>();
    let icicle_scalars = HostSlice::from_slice(&convert_scalars);

    let convert_points = points
        .iter()
        .map(|point| PointConversion::to_icicle(point))
        .collect::<Vec<_>>();

    let icicle_points = HostSlice::from_slice(&convert_points);

    let mut msm_results = DeviceVec::<curve::G1Projective>::cuda_malloc(1).unwrap();
    let stream = CudaStream::create().unwrap();
    cfg.ctx.stream = &stream;
    cfg.is_async = true;
    msm::msm(icicle_scalars, icicle_points, &cfg, &mut msm_results[..]).unwrap();

    let mut msm_host_result = vec![curve::G1Projective::zero(); 1];

    stream.synchronize().unwrap();
    msm_results.copy_to_host(HostSlice::from_mut_slice(&mut msm_host_result[..])).unwrap();

    stream.destroy().unwrap();
    let res = PointConversion::from_icicle(&msm_host_result[0]).unwrap();
    Ok(res)
}

fn point_times_5() -> BlsG1point {
    let x = BLS12381FieldElement::from_hex_unchecked(
        "32bcce7e71eb50384918e0c9809f73bde357027c6bf15092dd849aa0eac274d43af4c68a65fb2cda381734af5eecd5c",
    );
    let y = BLS12381FieldElement::from_hex_unchecked(
        "11e48467b19458aabe7c8a42dc4b67d7390fdf1e150534caadddc7e6f729d8890b68a5ea6885a21b555186452b954d88",
    );
    BlsG1point::new([x, y, FieldElement::one()])
}
