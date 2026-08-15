# Test fixtures

`marked-demo.txt` contains synthetic invisible characters for local testing:

- U+200B ZERO WIDTH SPACE
- U+2060 WORD JOINER
- U+200D ZERO WIDTH JOINER
- U+00A0 NO-BREAK SPACE
- U+E0061 UNICODE TAG CHARACTER

Safe mode removes U+200B, U+2060 and U+E0061 while leaving context-sensitive U+200D and U+00A0 for review.

Strict mode removes the context-sensitive joiner and normalizes the non-breaking space. The expected strict output is in `expected-strict.txt`.
