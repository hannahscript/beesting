# beesting 🐝

I am reading through the [MAL (make a lisp)](https://github.com/kanaka/mal) guide to make one in Rust

It is called beesting for some reason

Code sample:
```
(def! fibt (fun* (n a b) (if (< n 1) a (fibt (- n 1) b (+ a b))) ))
```
This defines a fibonacci function that will be tail-call optimized.
