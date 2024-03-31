# hurl-language-server-prototype

This repo is a prototype I put together to learn about the language server protocol. I worked on in sporatically over a couple days to understand some of the core concepts of [the spec](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/).

## Goals
- [x] Implement Serializing/Desializing of Requests/Responses/Notifications
- [x] Implement document synchronization for didOpen/didChange
- [x] Implement completion for asserts (jsonpath, header, etc.)
- [x] Implement hover to show documentation for jsonpath, header, etc.
- [x] Implement diagnostics for asserts

## Non-goals
- Well-structured code
- Spec-compliant
- Re-usable

## Lessons Learned
 Going into this, I knew that [tower-lsp](https://github.com/ebkalderon/tower-lsp) already existed but I wanted to start as minimally as possible. If I build something more serious, I'll definitely be checking out this library because looks like it does all the parsing/transport stuff for me.

 I used the [ropey](https://crates.io/crates/ropey) crate to store the content. I think was a good call because it gives to a good interface for finding line by character indexes. I didn't implement partial document syncing but I think it could be really good for that as well. In the future I think I'd introduce tree-sitter with the hurl grammar as well so I can do smarter queries on the content. String matching sucks but it worked for this iteration.

 The `include_str!` macro was super helpful for including markdown in the code that I didn't want to actually put in the source file 😎👍

 The spec is actually pretty easy to work with and once you get things hooked up, it isn't very difficult to add more.

 ## Features that could be cool to add in the future
 - Code action to add an `Asserts`, `Options`, etc. block to the hurl file
 - Completions for matchers (isCollection, isBoolean, etc.)
 - "Running the LSP in an OpenAPI spec mode"
   - Expose a setting when starting up the LSP to give either a whole json/yaml spec as content or a file that contains a spec
   - Completion items and diagnostics for verbs, paths, request bodies, assertions in response bodies.
- Command to run hurl file from language server including parameters to specify a variables file or individual variables
- Command to re-run last hurl file

## Trying it yourself (Not Recommended)
Start by building the binary with cargo.

#### Neovim snippet
Here's the snippet I added in Neovim to start the client and then an autocommand to connect to hurl files:
```lua
local lspclient = vim.lsp.start_client({
  name = "hurlsp",
  cmd = { "<PUT THE BATCH OF THE BUILT BINARY IN HERE>" },
  on_attach = on_attach,
})

vim.api.nvim_create_autocmd("FileType", {
  pattern = "hurl",
  callback = function ()
    vim.lsp.buf_attach_client(0, lspclient)
  end
})

```

#### VS Code plugin
This repo also contains a small VS Code plugin to start the language server as well. To run it:
- Copy binary into the hurl-ls folder (siblings with `src`)
- `npm install` inside hurl-ls folder
- `npm run pacakge`
- Open the hurl-ls folder in VS Code
- Use the Run menu to run the extension
- This should open another instance of VS Code where you can open a hurl file and hopefully see it working
