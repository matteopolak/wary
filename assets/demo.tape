Output assets/demo.gif

Require nvim
Require cargo

Set Shell "zsh"
Set FontSize 35
Set Width 1920
Set Height 1080

Hide
Type "rm examples/vhs/src/main.rs"
Enter
Type "touch examples/vhs/src/main.rs"
Enter
Type "nvim examples/vhs/src/main.rs"
Enter
Sleep 0.1
Type "i"
Show

Wait+Screen /Completed Indexing/

Set TypingSpeed 200ms
Type "use wary::Wary;"
Enter
Enter
Type "struct User {"
Enter
Type "username: String,"
Enter
Type "age: u8,"
Down
Enter
Enter
Type "fn main() {"
Escape
Type ":w"
Enter
Type "0"
Type "kkkkkk"
Type "i"

Set TypingSpeed 120ms
Enter
Type "#[derive(Wary)]"
Set TypingSpeed 50ms

Escape
Type "0"
Type "jj"
Type "i"
Enter
Up
Tab

Set TypingSpeed 120ms
Type "#[validate(length(bytes, 15..=30), lowercase(ascii))]"
Set TypingSpeed 50ms

Escape
Type "0"
Type "jj"
Type "i"
Enter
Up
Tab

#Set TypingSpeed 120ms
Type "#[validate(range(min = 0, max = 120))]"
#Escape

Sleep 10

Type ":q!"
Enter
