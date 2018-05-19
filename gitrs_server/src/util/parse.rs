pub fn parse_u32(input: &str, radix: u32) -> u32 {
    u32::from_str_radix(input, radix).unwrap()
}

named!(pub sha<&str, &str>, take!(40));
