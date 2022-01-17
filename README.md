# Subject Classifier

Library for classifying a commit by it's subject. Tries hard to recognize the
subject type according to the commit message. Supports [Conventional Commits Standard v1.0.0](https://www.conventionalcommits.org/en/v1.0.0)

```rust
use subject_classifier::Subject;

let subject = Subject::from("feat: Add a new feature XYZ");
println!("Icon: {}, scope {}, msg: {}",
        subject.icon(),
        subject.scope(),
        subject.description);
```
