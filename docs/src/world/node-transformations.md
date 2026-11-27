# Node Transformations

[`crate::protocol::node_transform::NodeTransform`] converts a node's `paramtype2` and `param2` orientation fields into a 3 × 3 basis used for local-space transforms.

`from_param_type2` handles the orientation families represented in the implementation: facedir variants, wallmounted variants, 24-step rotation, and four-way rotation. Unrecognized parameter types use the identity transform. `from_y_quarters` creates quarter-turn rotations; `from_facedir` creates one of the 24 facedir orientations.

`facedir` returns the stored facedir index. `transform_point` applies the basis to a point. `transform_bounds` transforms all eight corners of an axis-aligned box and returns the enclosing local-space minimum and maximum.

The type is a geometry helper, not a packet decoder. Its input values are already-decoded node parameters.
