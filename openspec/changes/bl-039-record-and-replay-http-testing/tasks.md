## 1. VCR Middleware Integration

- [ ] 1.1 Add reqwest-middleware and reqwest-vcr to Cargo.toml dev-dependencies
- [ ] 1.2 Create VCR middleware builder function
- [ ] 1.3 Integrate with existing reqwest::Client
- [ ] 1.4 Add VCR_MODE environment variable parsing

## 2. Header Sanitization

- [ ] 2.1 Create sanitization middleware
- [ ] 2.2 Strip Authorization, Cookie, x-api-key headers
- [ ] 2.3 Strip headers matching *-token, *-secret patterns
- [ ] 2.4 Log sanitized headers for audit

## 3. Git-Crypt Setup

- [ ] 3.1 Create tests/cassettes/ directory
- [ ] 3.2 Add .gitattributes with git-crypt filter
- [ ] 3.3 Initialize git-crypt in repository
- [ ] 3.4 Export symmetric key for CI

## 4. CI Integration

- [ ] 4.1 Update CI workflow to install git-crypt
- [ ] 4.2 Add git-crypt unlock step before tests
- [ ] 4.3 Verify encrypted files in CI logs are binary

## 5. Testing

- [ ] 5.1 Write sanitization test with mock server
- [ ] 5.2 Verify cassette does not contain Authorization header
- [ ] 5.3 Write replay determinism test
- [ ] 5.4 Verify zero network latency in replay mode