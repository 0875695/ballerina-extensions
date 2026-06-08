"annotation" @keyword
"as" @keyword
"ascending" @keyword
"break" @keyword
"by" @keyword
"check" @keyword
"checkpanic" @keyword
"class" @keyword
"client" @keyword
"collect" @keyword
(commit_expr) @keyword
"configurable" @keyword
"conflict" @keyword
"const" @keyword
"continue" @keyword
"descending" @keyword
"distinct" @keyword
"do" @keyword
"else" @keyword
"enum" @keyword
"equals" @keyword
"external" @keyword
"fail" @keyword
"final" @keyword
"flush" @keyword
"foreach" @keyword
"fork" @keyword
"from" @keyword
"function" @keyword
"group" @keyword
"if" @keyword
"import" @keyword
"in" @keyword
"is" @keyword
"isolated" @keyword
"join" @keyword
"let" @keyword
"limit" @keyword
"listener" @keyword
"lock" @keyword
"match" @keyword
"new" @keyword
"on" @keyword
"order" @keyword
"outer" @keyword
"panic" @keyword
"private" @keyword
"public" @keyword
"readonly" @keyword
"record" @keyword
"remote" @keyword
"resource" @keyword
"retry" @keyword
"return" @keyword
"returns" @keyword
"rollback" @keyword
"select" @keyword
"service" @keyword
"source" @keyword
"start" @keyword
"table" @keyword
"transaction" @keyword
(transactional_qual) @keyword
"trap" @keyword
"type" @keyword
"typeof" @keyword
"var" @keyword
"wait" @keyword
"where" @keyword
"while" @keyword
"worker" @keyword
"xmlns" @keyword

";" @delimiter
":" @delimiter

"=" @operator
"==" @operator
"!=" @operator
"<" @operator
"<=" @operator
">" @operator
">=" @operator
"+" @operator
"-" @operator
"*" @operator
"/" @operator
"%" @operator
"?" @operator

(type_desc) @type
(builtin_type_name) @type.builtin
(type_reference
  (qualified_identifier
    (module_prefix) @module
    (identifier) @type))
(type_reference
  (identifier) @type)
(type_defn (identifier) @type)
(class_defn (identifier) @type)
(enum_decl (identifier) @type)

(function_defn (identifier) @function)
(method_defn (identifier) @function)
(method_decl (identifier) @function)

(function_reference
  (qualified_identifier
    (module_prefix) @module
    (identifier) @function))
(function_reference
  (identifier) @function)

(method_call_expr
  (identifier) @function.method)

(remote_call_expr
  (identifier) @function.method)

(string_literal) @string
(back_tick_string) @string
(template_substitution
  "${" @punctuation.special
  "}" @punctuation.special)

(variable_reference_expr
  (qualified_identifier
    (module_prefix) @module
    (identifier) @variable))
(variable_reference_expr
  (identifier) @variable)

(binding_pattern
  (identifier) @variable)

(param
  (identifier) @variable)


(int_literal) @number
(floating_point_literal) @number
(boolean_literal) @constant.builtin
"null" @constant.builtin

(field_name) @property
(field_access_lvexpr (identifier) @property)
(field_access_expr (identifier) @property)
(named_arg (identifier) @property)

(annotation_attachment
  "@" @attribute
  (identifier) @type)

(annotation_attachment
  "@" @attribute
  (qualified_identifier
    (module_prefix) @module
    (identifier) @type))

(comment) @comment
