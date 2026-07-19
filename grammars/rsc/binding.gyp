{
  "targets": [
    {
      "target_name": "tree_sitter_rsc_binding",
      "include_dirs": [
        "<!@(node -p \"require('node-addon-api').include\")",
        "src"
      ],
      "dependencies": ["<!(node -p \"require('node-addon-api').gyp\")"],
      "sources": [
        "bindings/node/binding.cc",
        "src/parser.c"
      ],
      "conditions": [
        ["OS!='win'", {
          "cflags_c": ["-std=c11"],
          "cflags_cc": ["-std=c++17"]
        }]
      ]
    }
  ]
}
