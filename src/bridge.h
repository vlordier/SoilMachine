#pragma once
#include <cstdint>
#include <cstddef>

// Forward declaration of opaque sim pointer
struct Sim;

extern "C" {
  void* sim_create(uint32_t seed, int32_t dim_x, int32_t dim_y, const char* soil_file);
  void  sim_destroy(void* raw);
  void  sim_step_water(void* raw, int32_t n);
  void  sim_step_wind(void* raw, int32_t n);
  double sim_get_height(void* raw, int32_t x, int32_t y);
  void  sim_get_heightmap(void* raw, float* out);
  void  sim_get_color_map(void* raw, uint8_t* out, int32_t* len);
  void  sim_get_normal_x(void* raw, int32_t x, int32_t y, float* out);
  void  sim_get_normal_y(void* raw, int32_t x, int32_t y, float* out);
  void  sim_get_normal_z(void* raw, int32_t x, int32_t y, float* out);
  int32_t sim_get_dim_y(void* raw);
  void  sim_set_seed(void* raw, uint32_t seed);
  void  sim_set_nwater(void* raw, int32_t n);
  void  sim_set_nwind(void* raw, int32_t n);
  void  sim_set_scale(void* raw, int32_t s);
  int32_t sim_get_dim_x(void* raw);
  int32_t sim_get_dim_y(void* raw);
}
