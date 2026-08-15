Output demo/assets/injm.gif

Set Shell "zsh"
Set Width 1200
Set Height 650
Set FontSize 20
Set TypingSpeed 45ms
Set Padding 20
Set Theme "Catppuccin Mocha"

Hide
Type `rm -f demo/input.rs demo/output.md`
Enter
Type `printf 'fn main() {\n    // injm begin <hello-injm\n    println!("Hello from injm!");\n    // injm end\n}\n' > demo/input.rs`
Enter
Type `printf '%b\n' '# Demo of injm\n' '<!-- injm begin >hello-injm :indent=0 :trim :offset=1 -->' '\x60\x60\x60rust' '\x60\x60\x60\n<!-- injm end -->\n' > demo/output.md`
Enter
Type "clear"
Enter
Show

Type "bat demo/input.rs demo/output.md"
Enter
Sleep 4s

Type `injm inject --input demo/input.rs --output demo/output.md`
Enter
Sleep 2s

Hide
Type "clear"
Enter
Show

Type "bat demo/output.md"
Enter
Sleep 4s

Hide
Type `rm -f demo/input.rs demo/output.md`
Enter
