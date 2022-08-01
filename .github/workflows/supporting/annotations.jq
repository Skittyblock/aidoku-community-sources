def annotation_level(x):
  if x == "help" or x == "note" then 
    "notice" 
  elif x == "warning" then 
    "warning" 
  else 
    "error"
  end;

.[] 
  | select(.reason == "compiler-message" and .message.code and .message.spans[].is_primary)
  | .message
  | (.spans[] | select(.is_primary)) as $span
  | "::\(annotation_level(.level)) file=$WORKING_DIRECTORY/\($span.file_name),line=\($span.line_start),endLine=\($span.line_end),title=\(.message)::\(.rendered)"
