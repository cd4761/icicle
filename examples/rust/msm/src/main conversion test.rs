use lambdaworks_math::elliptic_curve::short_weierstrass::{
    curves::bls12_381::{
    curve::BLS12381Curve, 
    default_types::FrElement
},
point::{ShortWeierstrassProjectivePoint, PointFormat, Endianness},
};
use lambdaworks_math::unsigned_integer::element::UnsignedInteger;

type BLS_G1Point = ShortWeierstrassProjectivePoint<BLS12381Curve>;

fn main() {
// 16진수 문자열을 FrElement로 변환
let x = "0x18887b03b2d42285b44f942414a101611a18efbacedc7fb936e0f739413391fafee6263970338a40da768bb5ef6d9c2f";
let y = "0x1206091d8b64cc0528e9a07f4f51c003eab7ff8d93bf4efb530d68ea05150a55f415261d30477d16a9442d8d6bed3c3"


let g1_point = BLS_G1Point::from_affine(
    FieldElement::from_hex(&x).unwrap(),
    FieldElement::from_hex(&y).unwrap(),
)
.unwrap();

let reverse_x = reverse_endianness(x);
let reverse_y = reverse_endianness(y);

let g1_point_reverse = BLS_G1Point::from_affine(
    FieldElement::from_hex(&reverse_x).unwrap(),
    FieldElement::from_hex(&reverse_y).unwrap(),
)
.unwrap();


println!("g1_point: {:?}", g1_point);
println!("g1_point_reverse: {:?}", g1_point_reverse);
// println!("Reverted hex values: {:?}", hex_values_back);
}


fn reverse_endianness(hex_str: &str) -> Result<String, &'static str> {

if hex_str.len() % 2 != 0 {
    println!("hex: {:?}, {:?}", hex_str.len(), hex_str);
    return Err("Invalid hex string length");
}

let reversed = hex_str
    .as_bytes()
    .chunks(2)
    .rev()
    .map(|chunk| std::str::from_utf8(chunk).unwrap())
    .collect::<Vec<_>>()
    .join("");

if reversed.len() % 2 != 0 {
    
    println!("reversed: {:?}, {:?}", reversed.len(), hex_str);
    return Err("Invalid hex string length");
}

Ok(reversed)
}