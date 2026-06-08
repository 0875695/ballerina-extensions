(raw_template_expression
  (back_tick_string
    (back_tick_body) @injection.content))
(#set! injection.language "sql")

(xml_template_expression
  (back_tick_string
    (back_tick_body) @injection.content))
(#set! injection.language "xml")
