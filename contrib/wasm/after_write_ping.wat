;; Build: wat2wasm after_write_ping.wat -o after_write_ping.wasm
;; Install: copy `after_write_ping.wasm` to `.cantrik/plugins/`
(module
  (func (export "after_write_ping") (result i32)
    (i32.const 0)))
