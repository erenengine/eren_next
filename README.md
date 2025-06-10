# 에렌 엔진
에렌 엔진은 코드 중심의 게임 엔진입니다.

> ⚠️ 이 프로젝트는 현재 개발 중입니다. 주요 기능과 문서가 변경될 수 있습니다.

* TypeScript 버전의 경우, Rust 버전을 백엔드로 사용하고 TypeScript로는 로직을 구현합니다.
* Rust Wasm 타겟은 wgpu를 사용하며 WGSL로 쉐이더 코드를 작성합니다.
* 기타 타겟은 ash를 사용하며 GLSL로 쉐이더 코드를 작성합니다.

```
export VULKAN_SDK=$HOME/VulkanSDK/1.4.313.1/macOS
export DYLD_FALLBACK_LIBRARY_PATH=$VULKAN_SDK/lib
export VK_ICD_FILENAMES=$VULKAN_SDK/share/vulkan/icd.d/MoltenVK_icd.json
export VK_LAYER_PATH=$VULKAN_SDK/share/vulkan/explicit_layer.d
```

```
glslc sprite.vert -o sprite.vert.spv
glslc sprite.frag -o sprite.frag.spv
```

## TODO
- WGPU 버전을 2.0으로, 추후 Ash까지 지원하는 버전을 3.0으로

## 커뮤니티
- [네이버 카페](https://cafe.naver.com/erenengine)
- [디스코드](https://discord.gg/VyeJKK4c7J)
