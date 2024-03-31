### JSONPath assert

Check the value of a [JSONPath] query on the received HTTP body decoded as a JSON
document. JSONPath assert consists of the keyword `jsonpath` followed by a predicate
function and value.

Let's say we want to check this JSON response:

```plain
curl -v http://httpbin.org/json

< HTTP/1.1 200 OK
< Content-Type: application/json
...

{
  "slideshow": {
    "author": "Yours Truly",
    "date": "date of publication",
    "slides": [
      {
        "title": "Wake up to WonderWidgets!",
        "type": "all"
      },
       ...
    ],
    "title": "Sample Slide Show"
  }
}
```

With Hurl, we can write multiple JSONPath asserts describing the DOM content:


```hurl
GET http://httpbin.org/json
HTTP 200
[Asserts]
jsonpath "$.slideshow.author" == "Yours Truly"
jsonpath "$.slideshow.slides[0].title" contains "Wonder"
jsonpath "$.slideshow.slides" count == 2
jsonpath "$.slideshow.date" != null
jsonpath "$.slideshow.slides[*].title" includes "Mind Blowing!"
```

> Explain that the value selected by the JSONPath is coerced to a string when only
> one node is selected.

In `matches` predicates, metacharacters beginning with a backslash (like `\d`, `\s`) must be escaped.
Alternatively, `matches` predicate support [JavaScript-like Regular expression syntax] to enhance
the readability:

```hurl
GET https://sample.org/hello
HTTP 200
[Asserts]

# Predicate value with matches predicate:
jsonpath "$.date" matches "^\\d{4}-\\d{2}-\\d{2}$"
jsonpath "$.name" matches "Hello [a-zA-Z]+!"

# Equivalent syntax:
jsonpath "$.date" matches /^\d{4}-\d{2}-\d{2}$/
jsonpath "$.name" matches /Hello [a-zA-Z]+!/
```
