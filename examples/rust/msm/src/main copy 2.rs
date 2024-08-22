use lambdaworks_math::{
    field::element::FieldElement,
    polynomial::bivariate::BivariatePolynomial,
    elliptic_curve::short_weierstrass::curves::bls12_381::curve::BLS12381Curve,
};

use icicle_bn254::curve::{CurveCfg, G1Projective, G2CurveCfg, G2Projective, ScalarCfg};

fn convert_to_bivariate_polynomial(
    scalars_slice: &[ScalarCfg],
    points_slice: &[G1Projective],
    x_degree: usize,
    y_degree: usize,
) -> BivariatePolynomial<FieldElement<BLS12381Curve>> {
    // 변환된 데이터를 담을 벡터 초기화
    let mut coefficients: Vec<Vec<FieldElement<BLS12381Curve>>> =
        vec![vec![FieldElement::zero(); y_degree + 1]; x_degree + 1];

    // HostSlice의 데이터를 BivariatePolynomial의 계수로 변환
    for (i, scalar) in scalars_slice.iter().enumerate() {
        let x_index = i % (x_degree + 1);
        let y_index = i / (x_degree + 1);
        if y_index > y_degree {
            break;
        }
        coefficients[x_index][y_index] = FieldElement::from(*scalar);
    }

    // BivariatePolynomial 생성
    BivariatePolynomial::new(coefficients)
}

fn main() {
    let size: usize = 524288; // 예제 크기 설정s
    let x_degree = 5; // X 차수
    let y_degree = 5; // Y 차수
    let upper_size = 1 << (22);
    let upper_points = CurveCfg::generate_random_affine_points(upper_size);
    let g2_upper_points = G2CurveCfg::generate_random_affine_points(upper_size);
    let upper_scalars = ScalarCfg::generate_random(upper_size);

    // ICICLE의 HostSlice를 LambdaWorks의 BivariatePolynomial로 변환
    let bivariate_polynomial = convert_to_bivariate_polynomial(
        &upper_scalars[..size], // scalar 데이터를 가져옵니다
        &upper_points[..size], // point 데이터를 가져옵니다
        x_degree,
        y_degree,
    );

    println!("Bivariate Polynomial: {:?}", bivariate_polynomial);
}
