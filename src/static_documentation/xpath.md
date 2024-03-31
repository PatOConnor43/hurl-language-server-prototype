### XPath assert

Check the value of a [XPath] query on the received HTTP body decoded as a string (using the `charset` value in the
`Content-Type` header response). Currently, only XPath 1.0 expression can be used. Body assert consists of the
keyword `xpath` followed by a predicate function and value. Values can be string,
boolean or number depending on the XPath query.

Let's say we want to check this HTML response:

```plain
$ curl -v https://example.org

< HTTP/1.1 200 OK
< Content-Type: text/html; charset=UTF-8
...
<!doctype html>
<html>
  <head>
    <title>Example Domain</title>
    ...
  </head>

  <body>
    <div>
      <h1>Example</h1>
      <p>This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission.</p>
      <p><a href="https://www.iana.org/domains/example">More information...</a></p>
    </div>
  </body>
</html>
```

With Hurl, we can write multiple XPath asserts describing the DOM content:

```hurl
GET https://example.org
HTTP 200
Content-Type: text/html; charset=UTF-8
[Asserts]
xpath "string(/html/head/title)" contains "Example" # Check title
xpath "count(//p)" == 2                             # Check the number of <p>
xpath "//p" count == 2                              # Similar assert for <p>
xpath "boolean(count(//h2))" == false               # Check there is no <h2>  
xpath "//h2" not exists                             # Similar assert for <h2> 
```

XML Namespaces are also supported. Let's say you want to check this XML response:

```xml
<?xml version="1.0"?>
<!-- both namespace prefixes are available throughout -->
<bk:book xmlns:bk='urn:loc.gov:books'
         xmlns:isbn='urn:ISBN:0-395-36341-6'>
    <bk:title>Cheaper by the Dozen</bk:title>
    <isbn:number>1568491379</isbn:number>
</bk:book>
```

This XML response can be tested with the following Hurl file:

```hurl
GET http://localhost:8000/assert-xpath
HTTP 200
[Asserts]

xpath "string(//bk:book/bk:title)" == "Cheaper by the Dozen"
xpath "string(//*[name()='bk:book']/*[name()='bk:title'])" == "Cheaper by the Dozen"
xpath "string(//*[local-name()='book']/*[local-name()='title'])" == "Cheaper by the Dozen"

xpath "string(//bk:book/isbn:number)" == "1568491379"
xpath "string(//*[name()='bk:book']/*[name()='isbn:number'])" == "1568491379"
xpath "string(//*[local-name()='book']/*[local-name()='number'])" == "1568491379"
```

The XPath expressions `string(//bk:book/bk:title)` and `string(//bk:book/isbn:number)` are written with `bk` and `isbn`
namespaces.

> For convenience, the first default namespace can be used with `_`