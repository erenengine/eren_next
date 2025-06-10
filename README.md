# 에렌 엔진
에렌 엔진은 코드 중심의 게임 엔진입니다.

> ⚠️ 이 프로젝트는 현재 개발 중입니다. 주요 기능과 문서가 변경될 수 있습니다.

## Vulkan SDK 설정(맥)
```
export VULKAN_SDK=$HOME/VulkanSDK/1.4.313.1/macOS
export DYLD_FALLBACK_LIBRARY_PATH=$VULKAN_SDK/lib
export VK_ICD_FILENAMES=$VULKAN_SDK/share/vulkan/icd.d/MoltenVK_icd.json
export VK_LAYER_PATH=$VULKAN_SDK/share/vulkan/explicit_layer.d
```

## GLSL 컴파일
```
glslc sprite.vert -o sprite.vert.spv
glslc sprite.frag -o sprite.frag.spv
```

## 커뮤니티
- [네이버 카페](https://cafe.naver.com/erenengine)
- [디스코드](https://discord.gg/VyeJKK4c7J)
