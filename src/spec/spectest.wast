(module
  (type (;0;) (func))
  (type (;1;) (func (param i32)))
  (type (;2;) (func (param f32)))
  (type (;3;) (func (param f64)))
  (type (;4;) (func (param i32 f32)))
  (type (;5;) (func (param f64 f64)))
  (func (;0;) (type 0))
  (func (;1;) (type 1) (param i32))
  (func (;2;) (type 2) (param f32))
  (func (;3;) (type 3) (param f64))
  (func (;4;) (type 4) (param i32 f32))
  (func (;5;) (type 5) (param f64 f64))
  (table (;0;) 10 20 anyfunc)
  (memory (;0;) 1 2)
  (global (;0;) i32 (i32.const 666))
  (global (;1;) i64 (i64.const 666))
  (global (;2;) f32 (f32.const 0x1.4d4cccp+9 (;=666.6;)))
  (global (;3;) f64 (f64.const 0x1.4d4cccccccccdp+9 (;=666.6;)))
  (export "print" (func 0))
  (export "print_i32" (func 1))
  (export "print_f32" (func 2))
  (export "print_f64" (func 3))
  (export "print_i32_f32" (func 4))
  (export "print_f64_f64" (func 5))
  (export "table" (table 0))
  (export "memory" (memory 0))
  (export "global_i32" (global 0))
  (export "global_i64" (global 1))
  (export "global_f32" (global 2))
  (export "global_f64" (global 3)))
