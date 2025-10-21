use base85::{encode, decode};

fn main() {
    // Test encoding
    let test_data = b"Hello, World!";
    let encoded = encode(test_data);
    println!("Encoded: {}", encoded);
    println!("Valid chars in Z85: 0-9 a-z A-Z . - : + = ^ ! / * ? & < > ( ) [ ] {{ }} @ % $ #");
    
    // Test problematic characters from user's string
    let problematic = vec![
        ("I", "letter I"),
        ("y", "letter y"),
        ("o", "letter o"),
        ("#", "hash"),
        ("N", "letter N"),
        ("`", "backtick"),
        ("|", "pipe"),
        ("~", "tilde"),
    ];
    
    println!("\nTesting individual characters:");
    for (ch, name) in problematic {
        match decode(ch) {
            Ok(_) => println!("  {} ({}): VALID", ch, name),
            Err(e) => println!("  {} ({}): INVALID - {}", ch, name, e),
        }
    }
    
    // Try to decode user's string
    println!("\nTrying to decode user's verification_context:");
    let user_vc = "dm?KhIyo#NV`*|@b!l`WI$I(rL2PMbWlLpwAU!=GQ)OdvWpqnrc_|<!CLkzbXJsHhATc;8B3&#Zb98cHbY*9GB03^rb#!kcEFyDdV{&D5Uvp_^ZeeV5B05_lQ)OdvWpqnrc_J(#VP|C`U3~";
    match decode(user_vc) {
        Ok(data) => println!("  Success! Decoded {} bytes", data.len()),
        Err(e) => println!("  Error: {}", e),
    }
}
