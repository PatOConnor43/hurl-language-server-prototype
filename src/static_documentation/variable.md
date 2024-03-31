### Variable assert

```hurl
# Test that the XML endpoint return 200 pets 
GET https://example.org/api/pets
HTTP 200
[Captures]
pets: xpath "//pets"
[Asserts]
variable "pets" count == 200
```