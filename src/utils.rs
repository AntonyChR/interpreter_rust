pub fn is_letter(ch: &str) -> bool {
    let a = ch.chars().next().unwrap();
    return ('a'..='z').contains(&a) || ('A'..='Z').contains(&a) || a == '_';
}

pub fn is_digit(ch: &str) -> bool {
    let a = ch.chars().next().unwrap();
    return ('1'..='9').contains(&a) || a == '0';
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_is_letter() {
        use crate::utils::is_letter;
        let text = "qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM_";
        for ch in text.chars() {
            assert!(is_letter(&ch.to_string()), "fail to check {}", ch);
        }
    }

    #[test]
    fn test_is_digit() {
        use crate::utils::is_digit;
        let digits = "0123456789";
        for ch in digits.chars() {
            assert!(is_digit(&ch.to_string()), "fail to check {}", ch);
        }
    }
}
