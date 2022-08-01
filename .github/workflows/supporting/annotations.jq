def annotation_level(x):
  if x == "help" or x == "note" then 
    "notice" 
  elif x == "warning" then 
    "warning" 
  else 
    "failure"
  end;

map(
  select(.reason == "compiler-message" and .message.code and .message.spans[].is_primary)
  | .message
  | (.spans[] | select(.is_primary)) as $span
  | {
      title: .message,
      message: .rendered,
      annotation_level: annotation_level(.level),
      path: $span.file_name,
      start_line: $span.line_start,
      end_line: $span.line_end,
    }
  | if $span.line_start == $span.line_end then
      .start_column = $span.column_start
      | .end_column = $span.column_end
    else
      .
    end
)
