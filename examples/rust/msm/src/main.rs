use lambdaworks_math::elliptic_curve::short_weierstrass::curves::bls12_381::default_types::FrElement;
use lambdaworks_math::traits::{AsBytes, ByteConversion};
use lambdaworks_math::unsigned_integer::element::UnsignedInteger;
use msm::bipolynomial::BivariatePolynomial;

/// Converts a slice of FrElement to a Vec<u8>
fn fr_elements_to_host_slice(elements: &[FrElement]) -> Vec<u8> {
    elements.iter().flat_map(|element| element.to_bytes_le()).collect()
}

/// Converts a HostSlice back to a vector of FrElement
fn host_slice_to_fr_elements(bytes: &[u8], element_count: usize) -> Vec<FrElement> {
    // Manual creation of the element size using a default zeroed UnsignedInteger
    let element_size = UnsignedInteger::<4> { limbs: [0; 4] }.to_bytes_le().len();

    println!("Expected bytes length: {}", element_count * element_size);
    println!("Actual bytes length: {}", bytes.len());

    if bytes.len() < element_count * element_size {
        panic!("Insufficient bytes length for the number of elements");
    }

    (0..element_count)
        .map(|i| {
            let start = i * element_size;
            let end = start + element_size;
            let uint = UnsignedInteger::<4>::from_bytes_le(&bytes[start..end]).unwrap();
            FrElement::from_raw(uint)
        })
        .collect()
}

/// Converts a HostSlice of bytes back into a BivariatePolynomial
fn host_slice_to_bivariate_polynomial(
    host_slice: &[u8],
    x_degree: usize,
    y_degree: usize,
) -> BivariatePolynomial<FrElement> {
    let element_count = (x_degree) * (y_degree);
    let fr_elements = host_slice_to_fr_elements(host_slice, element_count);
  
    let mut coefficients: Vec<Vec<FrElement>> = vec![vec![]; x_degree];
    for i in 0..(x_degree) {
        for j in 0..(y_degree) {
            coefficients[i].push(fr_elements[i * (y_degree) + j].clone());
        }
    }

    let ref_coeffs: Vec<&[FrElement]> = coefficients.iter().map(|v| v.as_slice()).collect();
    BivariatePolynomial::new(&ref_coeffs)
}

fn main() {
    let bp = BivariatePolynomial::new(&[
        &[FrElement::from(2u64), FrElement::from(1u64)],
        &[FrElement::from(1u64), FrElement::from(1u64)],
    ]);

    let flattened_elements = bp.flatten_out();

    let coefficients_x_y: Vec<_> = flattened_elements
        .iter()
        .map(|coefficient| coefficient.representative())
        .collect();

    let bytes = fr_elements_to_host_slice(&flattened_elements);
    let host_slice: &[u8] = &bytes;

    println!("host_slice: {:?}", host_slice);
    println!("Original coefficients_x_y: {:?}", coefficients_x_y);

    // Corrected assertion for length
    assert!(host_slice.len() >= flattened_elements.len() * FrElement::from_raw(UnsignedInteger { limbs: [0; 4] }).as_bytes().len());

    let recovered_bp = host_slice_to_bivariate_polynomial(&host_slice[..], bp.x_degree, bp.y_degree);

    // println!("Original BivariatePolynomial: {:?}", bp);
    // println!("Recovered BivariatePolynomial: {:?}", recovered_bp.flatten_out());
    let flattend = recovered_bp.flatten_out();

    let recovered_coefficients_x_y: Vec<_> = flattend
        .iter()
        .map(|coefficient| coefficient.value())
        .collect();
    println!("Recovered coefficient: {:?}", recovered_coefficients_x_y);
}
