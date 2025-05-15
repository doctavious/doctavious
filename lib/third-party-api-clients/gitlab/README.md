# GitLab Client


## Implementation

I initially planned to have a client generated from GitLab's OpenAPI spec however I ran into several issues. The first one is that their [3.0 spec](https://gitlab.com/gitlab-org/gitlab/-/raw/master/doc/api/openapi/openapi.yaml) is currently woefully lacking endpoints. I then attempted to use their [Swagger 2.0](https://gitlab.com/gitlab-org/gitlab/-/raw/master/doc/api/openapi/openapi_v2.yaml) document however ran into issues with that as well. It's currently malformed which required manually correcting it. Once I did that I attempted to convert it to a OpenAPI v3 document using the [Swagger editor](https://editor.swagger.io/) however that again produce a malformed v3 document which required manual editing. Once I had that, I attempted to generate a rust client via [OpenAPI Generator CLI](https://github.com/OpenAPITools/openapi-generator) but it the generated spec didn't adhere to some rules which again required manual fixes. After all that, I was finally able to generate a client albeit not one I entirely liked, tho I might be too hasty with that conclusion. To top it all off one endpoint that I need (Merge Request comments aka notes) isn't even included.

I also attempted to generate a client with [Proginator](https://github.com/oxidecomputer/progenitor/tree/main), and unfortunately it failed with a fairly unhelpful error message. Not that it really matters because one of the endpoints we need isn't part of the spec.

With all that said current client is manually implemented focused purely on the endpoint we need.

## Pagination Details 
Pagination - https://docs.gitlab.com/api/rest/#pagination
Offset
page / per_page (default 20, max 100)

Keyset - supported by subset of resources. See https://docs.gitlab.com/api/rest/#supported-resources
pagination - keyset
per_page
order_by
sort

Link Header
link: <https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=1&per_page=3>; rel="prev", <https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=3&per_page=3>; rel="next", <https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=1&per_page=3>; rel="first", <https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=3&per_page=3>; rel="last"

Headers
```
x-next-page: 3
x-page: 2
x-per-page: 3
x-prev-page: 1
x-request-id: 732ad4ee-9870-4866-a199-a9db0cde3c86
x-runtime: 0.108688
x-total: 8
x-total-pages: 3
```