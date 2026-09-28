# Novelty Check: On-Device, Resource-Pressure-Aware Adaptive LOD for 3D Gaussian Splatting (as of 28 Sept 2026)

The gap still holds, but only in a narrower form. I found no 2024–2026 3DGS or NeRF paper that changes the Gaussian count or LOD tier live, at runtime, in response to measured free GPU memory, thermal state, or battery/power state. I also found nothing that breaks your claim outright. The general idea of lowering LOD when a device overheats is not new, though. Unity Adaptive Performance and at least one VR patent already do it for meshes and video.\[1\]\[2\] So the new part is bringing a closed-loop device-state controller to 3DGS LOD representations, not "thermal-aware adaptive rendering" as such.

## TL;DR

- **No claim-breaking paper found.** Every 3DGS LOD or streaming system I checked adapts to camera distance or projected size (Octree-GS, Hierarchical 3DGS, LODGE, Voyager, Atlas), network bandwidth (LapisGS, CAGS, SplatStream, MoQSplat), or a fixed, pre-set memory or Gaussian budget (FLoD, MoQSplat's M_max, GaussAnything's M = 200,000, Milef et al.'s budget-based CLoD). None reads live VRAM headroom, temperature, or battery. "Splats under Pressure" itself is an offline benchmark that emulates GPU tiers with power caps. It does not adapt anything.
- **Two partial threats to address head-on.** (1) GaussAnything (arXiv 2026) already uses measured frame-time slack in a runtime loop, but only to pace scene updates; the Gaussian budget stays fixed. (2) Outside 3DGS, Unity Adaptive Performance changes LOD bias live from device thermal warning level and trend, and a US patent picks VR video quality from monitored device temperature. A claim based on frame time alone is therefore the weakest part.
- **Narrow the framing to signal and representation, not to platform.** Claim: "the first closed-loop controller that maps measured device state (VRAM headroom, thermal/throttle state, power/battery) to a 3DGS Gaussian budget or LOD tier at runtime, evaluated against static, distance-based and fixed-budget 3DGS LOD." Present frame time as a supporting signal, not the novelty. Do not claim to be the first thermal- or power-aware adaptive renderer in general. Restricting the claim to "consumer laptops only" is unnecessary and would weaken it.

## Key Findings

1. **"Splats under Pressure" gives you your motivation, not your method.** Tajwar, Wuhrlin and Bhojan (NUS) emulate four GPU tiers (RTX 4090, 4070 Ti, 3070, 3050) on a single RTX 4090 using `nvidia-smi -pl`, `-lgc` and `-lmc`. They measure FPS, peak memory and power (via `nvidia-smi dmon`) at four fixed LapisGS-style LOD tiers of 0.58M, 1.83M, 2.79M and 3.45M Gaussians.\[3\] LOD is chosen by hand ("one can control the number of gaussians by lower or increasing the number of used layers"), and all layers are pre-loaded in GPU memory.\[4\]\[5\]\[6\] The authors' own future work calls for "LoD prediction and bandwidth-aware hierarchies". They do not mention runtime device-state control, which leaves your gap open and makes their paper your most natural anchor citation. Their data also shows why adaptation matters: an emulated RTX 3050 drops from 45.8 FPS at 0.58M splats to 19.7 FPS at 3.45M.\[3\]\[4\]
2. **Every major 3DGS LOD method adapts to geometry, not to the device.** Octree-GS picks levels by view;\[7\] Hierarchical 3DGS by distance;\[3\] LODGE by camera distance plus chunk loading; Voyager and Atlas by an LoD cut against a predefined pixel-size threshold.\[8\]\[9\]\[10\]\[11\]\[12\] Among them, FLoD comes closest in motivation ("rendered at varying levels of detail according to hardware capabilities"), but the level is picked once for a device's memory class, not re-picked as free memory changes.\[13\]
3. **The closest memory-side work is MoQSplat, and it uses a fixed budget.** It evicts enhancement layers from VRAM when the splat footprint passes a high watermark of 0.9 × M_max. M_max is a "fixed GPU memory budget", and the priority score comes from 6-DoF pose.\[14\] It never queries the device's actual free VRAM.\[15\] You need to cite it and draw this distinction explicitly.
4. **The closest frame-time work is GaussAnything.** It runs on a standalone headset with a fixed client capacity (M = 200,000 in experiments), and "the work allowance adapts within predefined limits according to recent display slack".\[16\] That is a live, measured frame-time loop, but it sets how fast updates are applied, not how many Gaussians are rendered.\[16\]
5. **Power- and thermal-aware 3DGS work exists, but it runs offline or only measures.** PowerGS (SIGGRAPH Asia 2025) minimizes combined display and rendering power under a quality constraint, but offline, with modelled power at an assumed 60 FPS.\[17\] Mobile-GS (ICLR 2026) reports thermal throttling on a Snapdragon 8 Gen 3, from 127 FPS cold start to 74 FPS steady state, and does not adapt to it.\[18\]\[19\] Both papers show the problem is real and unsolved.
6. **Outside 3DGS, the mechanism has prior art.** Unity Adaptive Performance's scalers (framerate, resolution, LOD, and others) change "by the device thermal warning level and thermal trend", and its documentation includes sample code that sets `QualitySettings.lodBias` from thermal events. US Patent 10,491,711 selects a VR stream quality level so that "decoding of the second data segment does not raise the temperature above the threshold temperature." NeRFlex (IEEE) is resource-aware for memory and compute, but configures offline.\[1\]\[2\]\[20\]\[21\] Treat these as related work that sets the bar. They do not break the 3DGS-specific claim.

## Details: Per-Paper Comparison

### A. The seed paper and its citers

| Paper | Authors / Venue / Year | Link | Adapts to | Live or offline | Closeness to your gap |
|---|---|---|---|---|---|
| Splats under Pressure: Exploring Performance–Energy Trade-offs in Real-Time 3DGS under Constrained GPU Budgets | M. F. Tajwar, A. Wuhrlin, A. Bhojan (NUS) · arXiv · 2026\[22\] | https://arxiv.org/abs/2604.07177 | Nothing. LOD tier set by hand; GPU tier emulated with power/clock caps\[6\] | Offline benchmark\[3\] | **Partial overlap** (same problem and signals measured, no adaptation) |
| Papers citing it | — | — | — | — | **Not verified.** Semantic Scholar's citations API returned HTTP 429 on every attempt. MoQSplat and GaussAnything do not cite it; PowerGS and Milef et al. predate it. Check Google Scholar "Cited by" by hand before submitting. |

### B. Named 3DGS streaming and LOD systems

| Paper | Authors / Venue / Year | Link | Adapts to | Live or offline | Closeness |
|---|---|---|---|---|---|
| LapisGS: Layered Progressive 3DGS for Adaptive Streaming | Y. Shi, S. Gasparini, G. Morin, W. T. Ooi\[23\] · 3DV 2025 (IEEE) | https://arxiv.org/abs/2408.14823 · https://ieeexplore.ieee.org/document/11125594/ | Network bandwidth ("bandwidth-aware streaming"); layers also enable view/distance-adaptive rendering\[24\]\[25\] | Layers built offline; layer choice at stream time | Different |
| Voyager: Real-Time City-Scale 3DGS on Resource-Constrained Devices | Zheng Liu et al. · arXiv · 2025 | https://arxiv.org/html/2506.02774 | Camera pose; LoD cut vs predefined projected-size threshold; cloud streams only newly visible Gaussians\[9\]\[26\] | Live (visibility), not device state | Different (targets resource-constrained devices, e.g. Orin, but no device-state signal)\[26\] |
| CAGS: Color-Adaptive Volumetric Video Streaming with Dynamic 3DGS | D. Yin, Y. Jin, J. Shi, I. Ding, M. Zhang, F. Wang, Z. Huang, C. Zhang, J. Liu, F. Dong\[27\] · SIGGRAPH 2026 Conf. Papers | https://arxiv.org/abs/2605.09279 · https://dl.acm.org/doi/10.1145/3799902.3811058 | Fluctuating network bandwidth (VQ-based LoDs + color restoration)\[28\] | Live (network) | Different |
| SplatStream: Fine Granular Scalable GS for Adaptive 3D Scene Streaming | Muhammad Talha + 5 co-authors\[29\] · arXiv · 2026 | https://arxiv.org/abs/2607.25971 | Network (MPEG-DASH sub-representations; spatial/temporal/fine-grained layers)\[30\] | Live (network) | Different |
| MoQSplat: Adaptive Progressive Streaming of 3DGS via MoQ | E. Artioli, M. Ghafari, M. T. Islam, F. Tashtarian, C. Rothenberg, C. Timmerer\[14\] · arXiv (IEEE ©) · 2026 | https://arxiv.org/abs/2609.18624 | 6-DoF pose priority (100 ms loop) + **fixed** VRAM budget M_max with 0.9× watermark eviction\[14\]\[15\] | Live, but against a fixed budget | **Partial overlap** (closest on memory; you must show your controller reads live headroom) |
| Octree-GS: Consistent Real-time Rendering with LOD-Structured 3D Gaussians | K. Ren, L. Jiang, T. Lu, M. Yu, L. Xu, Z. Ni, B. Dai · IEEE TPAMI 2025 | https://arxiv.org/abs/2403.17898 · https://ieeexplore.ieee.org/document/10993308/ | View / observation distance\[31\] | Live (geometry) | Different |
| A Hierarchical 3D Gaussian Representation for Real-Time Rendering of Very Large Datasets | B. Kerbl, A. Meuleman, G. Kopanas, M. Wimmer, A. Lanvin, G. Drettakis · ACM TOG 43(4) 2024\[3\] | Found as a reference in Splats under Pressure and Voyager; primary page not fetched | Distance (merges distant splats, refines as the camera approaches)\[3\] | Live (geometry) | Different |
| FLoD: Integrating Flexible Level of Detail into 3DGS for Customizable Rendering | Y. Seo, Y. S. Choi, H. S. Son, Y. Uh · ACM TOG 2025 | https://arxiv.org/abs/2408.12894 · https://dl.acm.org/doi/10.1145/3731430 | Hardware memory capacity (level chosen per device; demoed on a laptop with MX250 2 GB)\[32\]\[33\] | Chosen at deployment (static per device) | **Partial overlap** (same motivation, no live signal) |
| LODGE: Level-of-Detail Large-Scale Gaussian Splatting with Efficient Rendering | J. Kulhánek, M.-J. Rakotosaona, F. Manhardt, C. Tsalicoglou, M. Niemeyer, T. Sattler, S. Peng, F. Tombari · NeurIPS 2025 | https://arxiv.org/abs/2505.23158 | Camera distance + spatial chunk loading on memory-constrained devices (iPhone 13 mini, MacBook Air M3, Chromebook)\[10\]\[34\] | Live (geometry) | Different |
| Learning Fast 3DGS Rendering using Continuous Level of Detail | N. Milef, D. Seyb, T. Keeler, T. Nguyen-Phuoc, A. Božič, S. Kondguli, C. S. Marshall\[35\] (Meta) · CGF 44(2) e70069 / Eurographics 2025 | https://doi.org/10.1111/cgf.70069 | The CGF abstract says "An arbitrary LOD level can then be selected by simply rendering the first N splats" and that the method "facilitates many performance-dependent rendering applications such as distance-based LOD, foveated rendering, and budget-based rendering"; code exposes a static `--max_splats` cap "for when there is not enough VRAM" | Budget appears user-set; the abstract doesn't say whether frame time sets it, so the full text still needs checking | **Partial overlap (verify)**: the most important paper to read in full |
| Atlas: Algorithm-Hardware Co-Design for On-Device City-Scale 3DGS in VR | He Zhu et al. (SJTU) · arXiv (cs.AR) · 2026 | https://arxiv.org/abs/2609.02352 | Pose + predefined LoD threshold τ*; demand paging/eviction of Gaussians\[8\]\[12\] | Live (visibility/demand) | Different (runtime Gaussian management §6 not fully read) |
| GaussAnything: Semantic Intent-Driven Refinement of Evolving Gaussian Scenes for Standalone VR | D. Maliukov, T. Kozlov, D. Plotnikov, M. Altamirano Cabrera, D. Tsetserukou\[16\] · arXiv · 2026 | https://arxiv.org/html/2609.13859 | Semantic query reallocates a **fixed** budget (M = 200,000); frame-time slack paces update work\[16\] | Live frame-time loop, but not for Gaussian count | **Partial overlap** (closest frame-time feedback) |

### C. Power, thermal and mobile 3DGS / NeRF

| Paper | Authors / Venue / Year | Link | Adapts to | Live or offline | Closeness |
|---|---|---|---|---|---|
| PowerGS: Display-Rendering Power Co-Optimization for Foveated Radiance-Field Rendering | W. Lin, S. Kondguli, C. Marshall, Y. Zhu · SIGGRAPH Asia 2025\[17\] | https://arxiv.org/abs/2509.21702 | Power (modelled) under a quality constraint; up to 86% power reduction\[36\] | Offline | Partial (power objective, no runtime battery signal) |
| Mobile-GS: Real-time Gaussian Splatting for Mobile Devices | X. Du, Y. Wang, K. Zhan, X. Yu · ICLR 2026\[37\] | https://arxiv.org/abs/2603.11531 | None; reports cold-start 127 FPS vs thermally throttled 74 FPS\[19\] | Measurement only | Different (strong motivation evidence) |
| VRSplat: Fast and Robust Gaussian Splatting for VR | X. Tu, L. Radl, M. Steiner, M. Steinberger, B. Kerbl, F. de la Torre · Proc. ACM CGIT 8(1) 2025 (arXiv:2505.10144) | https://dl.acm.org/doi/full/10.1145/3728311 | Gaze (foveated); "achieving 72+ FPS while eliminating popping and stereo-disrupting floaters" (RTX 4090 → Quest 3) | Live (gaze) | Different |
| RAVE: Rate-Adaptive Visual Encoding for 3DGS | H.-N. Tran, F. Di Sario, G. Spadaro, G. Valenzise, E. Tartaglione · arXiv:2512.07052 · 2025 | https://arxiv.org/html/2512.07052 | Target rate "determined by constraints such as network bandwidth, storage budget, or hardware capabilities";\[38\] it notes existing approaches "operate at fixed rates, limiting adaptability to varying bandwidth and device constraints" | Rate chosen per deployment | Different |
| WebSplatter: Cross-Device Efficient GS in Web Browsers via WebGPU | Yudong Han + 5 co-authors · arXiv:2602.03207 · 2026 (Zenodo code archive ties it to ACM Multimedia 2026) | https://arxiv.org/pdf/2602.03207 | None (lower VRAM, 1.20 GB on garden, to avoid crashes; reports "1.2× to 4.5× speedups over state-of-the-art web viewers") | Static optimization | Different |
| A LoD of Gaussians: Out-of-Core Training and Rendering | F. Windisch, T. Köhler, L. Radl, M. D'Urso, M. Steiner, D. Schmalstieg, M. Steinberger (TU Graz) · arXiv:2507.01110 v4 (17 Feb 2026) | https://arxiv.org/html/2507.01110v4 | View-dependent LoD with out-of-core streaming and caching;\[39\] motivated by the fact that "rendering remains fundamentally limited by GPU memory, as all visible chunks must reside in VRAM simultaneously" | Live (view) | Different |
| NeRFlex: Resource-aware Real-time High-quality Rendering of Complex Scenes on Mobile | Z. Wang, Y. Zhu (SJTU) · IEEE · 2025 | https://arxiv.org/abs/2504.03415 · https://ieeexplore.ieee.org/document/11183800/ | Device memory and compute constraints (profiler + dynamic programming over configs)\[21\] | Offline configuration | Partial (resource-aware, not live) |

### D. Outside 3DGS: mesh LOD, mobile VR, game engines

| Work | Source / Year | Link | Adapts to | Live or offline | Closeness |
|---|---|---|---|---|---|
| Unity Adaptive Performance (with Samsung / Android ADPF provider) | Unity docs; Android Developers | https://docs.unity3d.com/Packages/com.unity.adaptiveperformance@2.0/manual/user-guide.html · https://developer.android.com/games/engines/unity/unity-adpf | Device thermal warning level and trend; scalers for framerate, resolution, LOD, shadows, etc.\[2\]\[40\] | **Live, measured thermal** | Same mechanism, different representation (industry prior art, not a paper) |
| US 10,491,711: Adaptive streaming of VR data | USPTO patent | https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/10491711 | Monitored device temperature → quality level for next segment\[1\] | **Live, measured thermal** | Same mechanism for VR video, not 3DGS |
| US 11,474,591: Fine-grain GPU power management and scheduling for VR | USPTO patent | https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/11474591 | GPU temperature approaching threshold → lower GPU power state for low-priority intervals\[41\] | Live, thermal (power state, not LOD) | Partial |
| Improving Mobile Gaming Performance through Cooperative CPU-GPU Thermal Management | A. Prakash, H. Amrouch, M. Shafique, T. Mitra, J. Henkel · Proc. DAC 2016, 47:1–47:6 | https://dl.acm.org/doi/10.1145/2897937.2898031 | Thermal constraint → CPU/GPU frequency (control-theoretic)\[42\] | Live (DVFS, not content) | Partial |
| Power and Thermal Analysis of Commercial Mobile Platforms | G. Bhat et al. · IEEE DATE 2019 (arXiv:1904.09814) | https://arxiv.org/pdf/1904.09814 | None for content; throttling cuts Paper.io from 35 to 23 FPS (34%); proposes a governor that "throttles select applications without affecting other apps" | Measurement | Motivation only |
| Adaptive Graphical Settings Optimization for Energy-Efficient Mobile 3D Rendering | IEEE (NSF PAR 10679664) | https://par.nsf.gov/biblio/10679664-adaptive-graphical-settings-optimization-energy-efficient-mobile-rendering | Application behaviour; Pixel 8a "Boat Attack"; 43% energy reduction\[43\] | Static graphics scaling | Partial |
| Seeing enough: non-reference perceptual resolution selection for power-efficient client-side rendering | arXiv · 2026 | https://arxiv.org/pdf/2604.07959 | Content/motion → resolution, to save power\[44\] | Live (content) | Partial |
| Streaming of rendered content with adaptive frame rate and resolution | Y. Liu, J. G. March, R. K. Mantiuk · SIGGRAPH 2026 | https://arxiv.org/pdf/2605.10995 | Motion velocity, content, bandwidth\[45\] | Live (network/content) | Different |

## Verdict

**Does the gap hold? Yes, for 3DGS.** No paper I found closes the loop from measured VRAM headroom, thermal/throttle state, or battery/power state to the Gaussian count or LOD tier. Every 3DGS "budget" I found is a number the developer sets (FLoD's level, MoQSplat's M_max, GaussAnything's M, Milef et al.'s N or `--max_splats`, RAVE's target rate). **No paper was flagged as claim-breaking.**

**How to narrow the framing:**
- **Keep:** thermal/throttle-state and live VRAM-headroom adaptation of 3DGS LOD. These are the least-covered parts and the most defensible.
- **Keep, with caution:** battery/power state. PowerGS covers power offline, so your novelty is the runtime signal.
- **Demote:** frame-time degradation. Frame-time feedback is standard in engines, GaussAnything already uses display slack at runtime, and Milef et al.'s budget-based CLoD may include it (verify). Use it as an input, not the headline.
- **Don't narrow to "consumer laptops only".** The platform isn't where the novelty lies. Splats under Pressure's CUDA desktop/laptop tiers are a convenient testbed, but a device-agnostic controller is the stronger claim.
- **Never claim** "first thermal-aware adaptive LOD" in general. Unity Adaptive Performance and US 10,491,711 are counter-examples.\[1\]\[2\]

**Suggested claim wording:** "To our knowledge, this is the first 3DGS renderer that adapts its Gaussian budget or LOD tier at runtime to measured device state (available GPU memory, thermal/throttle status, and power state), rather than to viewpoint, network bandwidth, or a fixed developer-set budget."

## Recommendations

1. **Before submitting, close three verification holes:** (a) read Milef et al. §"budget-based rendering" to check whether the budget comes from frame time; (b) read Atlas §6 (Runtime Gaussian Management); (c) pull the Google Scholar "Cited by" list for arXiv 2604.07177 by hand.
2. **Evaluate against the right baselines:** a static per-device tier (FLoD-style), distance LOD (Octree-GS/LODGE-style), a fixed-VRAM-budget eviction policy (MoQSplat-style), and a Unity-style thermal-warning step-down. Beating the last two is what shows your contribution is more than engineering.
3. **Reuse the seed paper's pressure protocol.** Induce pressure with `nvidia-smi` power and clock caps and log with `nvidia-smi dmon`, as Splats under Pressure does.\[3\]\[6\] Add real sustained-load throttling runs (Mobile-GS-style cold-start vs steady-state) so reviewers can't dismiss emulation. Pith's automated review of that paper already flags that emulation "risks missing real architectural differences in lower-tier hardware".\[46\]
4. **Make the 3DGS-specific parts your technical contribution:** a Gaussian-count→VRAM/frame-time predictor, hysteresis so tiers don't oscillate, and seamless tier switches using prefix-ordered representations (LapisGS layers or CLoD first-N ordering).

## Caveats

- Several key comparators are 2026 arXiv preprints that have not been peer-reviewed (MoQSplat, Atlas, GaussAnything, SplatStream). Their final versions may add runtime signals.
- The list of papers citing Splats under Pressure could not be retrieved. A paper posted after April 2026 could exist that I did not see.
- Milef et al. and Atlas were not read in full. My "partial" and "different" ratings for them rest on abstracts, code READMEs and partial text.
- Unity Adaptive Performance and the patents are industry and patent prior art, not peer-reviewed papers. Cite them in related work anyway, because reviewers will know them.
- I did not fetch the Hierarchical 3DGS primary page. Its entry comes from references in two papers I did read.

## Sources

1. [Adaptive streaming of virtual reality data](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/10491711)
2. [Unity Adaptive Performance and Android provider | Android Developers](https://developer.android.com/games/engines/unity/unity-adpf)
3. [Splats under Pressure: Exploring Performance–Energy Trade-offs in Real-Time 3D Gaussian Splatting under Constrained GPU Budgets](https://arxiv.org/html/2604.07177v1)
4. [Splats under Pressure: Exploring Performance-Energy Trade-offs in Real-Time 3D Gaussian Splatting under Constrained GPU Budgets](https://arxiv.org/pdf/2604.07177)
5. [Splats under Pressure: Exploring Performance–Energy Trade-offs in Real-Time 3D Gaussian Splatting under Constrained GPU Budgets](https://arxiv.org/html/2604.07177)
6. [(PDF) Splats under Pressure: Exploring Performance-Energy Trade-offs in Real-Time 3D Gaussian Splatting under Constrained GPU Budgets](https://www.researchgate.net/publication/403641897_Splats_under_Pressure_Exploring_Performance-Energy_Trade-offs_in_Real-Time_3D_Gaussian_Splatting_under_Constrained_GPU_Budgets)
7. [arxiv.org](https://arxiv.org/pdf/2403.17898v1)
8. [Atlas: Algorithm-Hardware Co-Design for On-Device City-Scale 3D Gaussian Splatting in VR](https://arxiv.org/html/2609.02352)
9. [Voyager: Real-Time City-Scale 3D Gaussian Splatting on Resource-Constrained Devices](https://arxiv.org/html/2506.02774)
10. [\[2505.23158\] LODGE: Level-of-Detail Large-Scale Gaussian Splatting with Efficient Rendering](https://arxiv.org/abs/2505.23158)
11. [Octree-GS: Towards Consistent Real-time Rendering with LOD-Structured 3D Gaussians](https://city-super.github.io/octree-gs/)
12. [Atlas: Algorithm-Hardware Co-Design for On-Device City-Scale 3D Gaussian Splatting in VR](https://arxiv.org/pdf/2609.02352)
13. [Paper page - FLoD: Integrating Flexible Level of Detail into 3D Gaussian Splatting for Customizable Rendering](https://huggingface.co/papers/2408.12894)
14. <https://arxiv.org/html/2609.18624>
15. [MoQSplat: Adaptive Progressive Streaming of 3D Gaussian Splatting via MoQ](https://arxiv.org/pdf/2609.18624)
16. [GaussAnything: Semantic Intent-Driven Refinementof Evolving Gaussian Scenes for Standalone VR](https://arxiv.org/html/2609.13859)
17. <https://arxiv.org/html/2509.21702>
18. [Mobile-GS: Real-time Gaussian Splatting for Mobile Devices](https://arxiv.org/html/2603.11531)
19. [Mobile-GS: Real-time Gaussian Splatting for Mobile Devices — Lacuna](https://lacuna.tiptreesystems.com/work/mobile-gs-real-time-gaussian-splatting-for-mobile-devices/wrk_ef6f4e9901bf43ddb1c9c51408998256)
20. [Adaptive Performance user guide | Adaptive Performance | 2.2.4](https://docs.unity.cn/Packages/com.unity.adaptiveperformance@2.2/manual/user-guide.html)
21. [NeRFlex: Resource-aware Real-time High-quality Rendering of Complex Scenes on Mobile Devices](https://arxiv.org/pdf/2504.03415)
22. [arxiv.org](https://arxiv.org/abs/2604.07177)
23. [PD-4DGS:Progressive Decomposition of 4D Gaussian Splatting for Bandwidth-Adaptive Dynamic Scene Streaming](https://arxiv.org/pdf/2605.11427)
24. [\[2408.14823v2\] LapisGS: Layered Progressive 3D Gaussian Splatting for Adaptive Streaming](https://arxiv.org/abs/2408.14823v2)
25. [LapisGS: Layered Progressive 3D Gaussian Splatting for Adaptive Streaming](https://openreview.net/pdf?id=470WxVD1L3)
26. [\[Literature Review\] Voyager: Real-Time Splatting City-Scale 3D Gaussians on Your Phone](https://www.themoonlight.io/en/review/voyager-real-time-splatting-city-scale-3d-gaussians-on-your-phone)
27. [CAGS: Color-Adaptive Volumetric Video Streaming with Dynamic 3D Gaussian Splatting](https://arxiv.org/pdf/2605.09279)
28. [CAGS: Color-Adaptive Volumetric Video Streaming with Dynamic 3D Gaussian Splatting](https://arxiv.org/html/2605.09279)
29. [\[2607.25971\] SplatStream: Fine Granular Scalable Gaussian Splatting for Adaptive 3D Scene Streaming](https://arxiv.org/abs/2607.25971)
30. [SplatStream: Fine Granular Scalable Gaussian Splatting for Adaptive 3D Scene Streaming](https://arxiv.org/pdf/2607.25971)
31. [3DGS.zip: A survey on 3D Gaussian Splatting Compression Methods](https://arxiv.org/pdf/2407.09510)
32. [FLoD: Integrating Flexible Level of Detail into 3D Gaussian Splatting for Customizable Rendering](https://3dgs-flod.github.io/flod/)
33. [FLoD: Integrating Flexible Level of Detail into 3D Gaussian Splatting for Customizable Rendering](https://paperreading.club/page?id=247937)
34. [\[Literature Review\] LODGE: Level-of-Detail Large-Scale Gaussian Splatting with Efficient Rendering](https://www.themoonlight.io/en/review/lodge-level-of-detail-large-scale-gaussian-splatting-with-efficient-rendering)
35. [Learning Fast 3D Gaussian Splatting Rendering using Continuous Level of Detail](https://darioseyb.com/publication/milef-2025-clod/)
36. <https://arxiv.org/abs/2509.21702>
37. [RoofGS: Roofline-Guided End-to-End Acceleration of 3D Gaussian Splatting](https://arxiv.org/pdf/2608.15785)
38. [RAVE: Rate-Adaptive Visual Encoding for 3D Gaussian Splatting](https://arxiv.org/html/2512.07052)
39. [A LoD of Gaussians: Out-of-Core Training and Rendering for Seamless Ultra-Large Scene Reconstruction](https://arxiv.org/html/2507.01110v4)
40. [Optimize game performance with Adaptive Performance in Unity](https://developer.samsung.com/codelab/gamedev/adaptive-performance-unity.html)
41. [Fine-grain GPU power management and scheduling for virtual reality applications](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/11474591)
42. [Improving mobile gaming performance through cooperative CPU-GPU thermal management | Proceedings of the 53rd Annual Design Automation Conference](https://dl.acm.org/doi/10.1145/2897937.2898031)
43. <https://par.nsf.gov/biblio/10679664-adaptive-graphical-settings-optimization-energy-efficient-mobile-rendering>
44. [Seeing enough: non-reference perceptual resolution selection for power-efficient client-side rendering](https://arxiv.org/pdf/2604.07959)
45. [Streaming of rendered content with adaptive frame rate and resolution](https://arxiv.org/pdf/2605.10995)
46. [Splats under Pressure: Exploring Performance-Energy Trade-offs in Real-Time 3D Gaussian Splatting under Constrained GPU Budgets · Pith](https://pith.science/paper/2604.07177)
