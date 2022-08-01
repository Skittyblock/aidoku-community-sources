def annotation_level(x):
    if x == "help" or x == "note" then 
        "notice" 
    elif x == "warning" then 
        "warning" 
    else 
        "failure"
    end;

map(
    select(
        .reason == "compiler-message" and .message.code != null and .message.spans[].is_primary == true
    )
) 
| map(
    { 
        title: .message.message, 
        message: .message.rendered, 
        annotation_level: annotation_level(.message.level),
    } 
    + (
        .message.spans[] 
        | select(.is_primary = true) 
        | { 
            path: .file_name, 
            start_line: .line_start, 
            end_line: .line_end, 
            start_column: .column_start, 
            end_column: .column_end 
          }
      )
) 
| map_values(if .start_line != .end_line then del(.start_column, .end_column) else . end)
