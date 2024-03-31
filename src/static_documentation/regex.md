### Regex assert

Check that the HTTP received body, decoded as text, matches a regex pattern.

```hurl
GET https://sample.org/hello
HTTP 200
[Asserts]
regex "^(\\d{4}-\\d{2}-\\d{2})$" == "2018-12-31"
# Same assert as previous using regex literals
regex /^(\d{4}-\d{2}-\d{2})$/ == "2018-12-31"
```

The regex pattern must have at least one capture group, otherwise the
assert will fail. The assertion is done on the captured group value. When the regex pattern is a double-quoted string, 
metacharacters beginning with a backslash in the pattern (like `\d`, `\s`) must be escaped; literal pattern enclosed by
`/` can also be used to avoid metacharacters escaping.