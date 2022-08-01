def count(s): reduce s as $_ (0; . + 1);
def count(stream; cond): count(stream | cond // empty);

.[]
  | select(.reason == "compiler-message" and .message.code) 
  | "## Results
| Message level           | Amount                                                          |
| ----------------------- | --------------------------------------------------------------- |
| Internal compiler error | \(count(.; .message.level == "error: internal compiler error")) |
| Error                   | \(count(.; .message.level == "error"))                          |
| Warning                 | \(count(.; .message.level == "warning"))                        |
| Note                    | \(count(.; .message.level == "note"))                           |       
| Help                    | \(count(.; .message.level == "help"))                           |

## Versions
- \($RUSTC_VERSION)
- \($CARGO_VERSION)
- \($CLIPPY_VERSION)
"
