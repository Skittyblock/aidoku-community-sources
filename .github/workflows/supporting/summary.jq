def count(s): reduce s as $_ (0;.+1);
def count(stream; cond): count(stream | cond // empty);

.[]
  | select(.reason == "compiler-message" and .message.code) 
  | count(.; .message.level == "help") as $help_count
  | count(.; .message.level == "note") as $note_count
  | count(.; .message.level == "warning") as $warning_count
  | count(.; .message.level == "error") as $error_count
  | count(.; .message.level == "error: internal compiler error") as $ice_count
  | "## Results
| Message level           | Amount                |
| ----------------------- | --------------------- |
| Internal compiler error | \($ice_count)         |
| Error                   | \($error_count)       |
| Warning                 | \($warning_count)     |
| Note                    | \($note_count)        |
| Help                    | \($help_count)        |

## Versions
- \($RUSTC_VERSION)
- \($CARGO_VERSION)
- \($CLIPPY_VERSION)
"
