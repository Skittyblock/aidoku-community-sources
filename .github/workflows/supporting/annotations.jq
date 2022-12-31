def annotation_level(x):
  if x == "help" or x == "note" then 
    "notice" 
  elif x == "warning" then 
    "warning" 
  else 
    "error"
  end;

def sanitize(x):
    x 
    | gsub("%"; "%25")
    | gsub("\n"; "%0A")
    | gsub("\r"; "%0D");

.[] 
  | select(.reason == "compiler-message" and .message.code and .message.spans[].is_primary)
  | .message
  | (.spans[] | select(.is_primary)) as $span
  | "::\(annotation_level(.level)) "
    + "file=\($WORKING_DIRECTORY)/\($span.file_name | gsub("\\\\"; "/")),"
    + "line=\($span.line_start),"
    + "endLine=\($span.line_end),"
    + "col=\($span.column_start),"
    + "endColumn=\($span.column_end),"
    + "title=\(.message)::\(sanitize(.rendered))"
