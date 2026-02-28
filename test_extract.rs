fn extract_base_command(cmd: &str) -> &str {
    let trimmed = cmd.trim();
    let first_word = trimmed.split_whitespace().next().unwrap_or("");
    first_word.rsplit('/').next().unwrap_or(first_word).rsplit('\\').next().unwrap_or(first_word)
}

fn extract_all_commands(command: &str) -> Vec<&str> {
    let mut commands = Vec::new();
    let mut rest = command;
    while !rest.is_empty() {
        let separators: &[&str] = &["&&", "||", "|", ";"];
        let mut earliest_pos = rest.len();
        let mut earliest_len = 0;
        for sep in separators {
            if let Some(pos) = rest.find(sep) {
                if pos < earliest_pos {
                    earliest_pos = pos;
                    earliest_len = sep.len();
                }
            }
        }
        let segment = &rest[..earliest_pos];
        let base = extract_base_command(segment);
        if !base.is_empty() {
            commands.push(base);
        }
        if earliest_pos + earliest_len >= rest.len() {
            break;
        }
        rest = &rest[earliest_pos + earliest_len..];
    }
    commands
}

fn main() {
    let cmd = r#"curl -s "https://quote.eastmoney.com/unify/r/1.601919" -w "\n""#;
    println!("{:?}", extract_all_commands(cmd));
}
