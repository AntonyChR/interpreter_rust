pub fn is_letter(ch: &str) -> bool {
    // TODO: if the input does not end with ";" the chars().next() method tries to access 
    // the next character that does not exist and panics
    match ch.chars().next() {
        Some(value) => ('a'..='z').contains(&value) || ('A'..='Z').contains(&value) || value == '_',
        None => false
    }

}

pub fn is_digit(ch: &str) -> bool {
    match ch.chars().next() {
        Some(value) => ('1'..='9').contains(&value) || value == '0',
        None => false
    }
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
