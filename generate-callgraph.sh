cargo rustc --example safe --release -- --emit=llvm-ir
cat target/release/examples/safe*.ll | opt -analyze -dot-callgraph
dot -Tsvg -ocallgraph.svg '<stdin>.callgraph.dot'
rm '<stdin>.callgraph.dot'
rm target/release/examples/safe*.ll
