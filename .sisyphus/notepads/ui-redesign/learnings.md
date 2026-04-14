## Border Audit Findings
- Found 15 occurrences of Borders::ALL and 4 of Block::default().borders() in src/ui/.
- Most content blocks use Borders::ALL, which contradicts the 'Modern/Minimal' aesthetic.
- Identified a need to reduce or remove borders from lists, inputs, and help text.
- Established DesignTokens for consistent spacing and layout.

