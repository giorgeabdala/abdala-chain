(module
 (type $0 (func (param i32 i32)))
 (type $1 (func (param i32 i32) (result i32)))
 (import "env" "memory" (memory $mimport$0 2))
 (import "env" "console_log" (func $fimport$0 (param i32 i32)))
 (global $global$1 i32 (i32.const 65549))
 (global $global$2 i32 (i32.const 65552))
 (data $0 (i32.const 65536) "hello, world!")
 (export "add" (func $0))
 (export "__data_end" (global $global$1))
 (export "__heap_base" (global $global$2))
 (func $0 (param $0 i32) (param $1 i32) (result i32)
  (call $fimport$0
   (i32.const 65536)
   (i32.const 13)
  )
  (i32.add
   (local.get $0)
   (local.get $1)
  )
 )
)