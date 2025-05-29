# eren_next
다음 버전의 에렌엔진을 작업하는 소스코드 저장소입니다.

* TypeScript 버전의 경우, Rust 버전을 백엔드로 사용하고 TypeScript로는 로직을 구현합니다.
* Rust Wasm 타겟은 wgpu를 사용하며 WGSL로 쉐이더 코드를 작성합니다.
* 기타 타겟은 ash를 사용하며 GLSL로 쉐이더 코드를 작성합니다.
