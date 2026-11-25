// SoilMachine C++ FFI bridge for Rust
// Wraps the simulation engine behind an opaque pointer.

#include <cstdint>
#include <cstddef>
#include <string>
#include <vector>
#include <map>
#include <deque>
#include <random>
#include <functional>
#include <algorithm>
#include <cmath>
#include <iostream>
#include <fstream>
// ─── OpenGL stubs (no-op for data-only simulation) ───────────────────────
unsigned GL_ARRAY_BUFFER    = 0x8E01;
unsigned GL_MAP_WRITE_BIT   = 0x0001;
unsigned GL_MAP_PERSISTENT_BIT = 0x0040;
unsigned GL_MAP_COHERENT_BIT   = 0x0080;
unsigned GL_TRIANGLES      = 0x0004;
unsigned GL_DEPTH_COMPONENT  = 0x1902;
unsigned GL_FLOAT           = 0x1406;
unsigned GL_ELEMENT_ARRAY_BUFFER = 0x8DE1;
unsigned GL_DRAW_INDIRECT_BUFFER = 0x8F3F;
unsigned GL_STATIC_DRAW    = 0x88E0;
unsigned GL_UNSIGNED_INT   = 0x1405;
typedef unsigned int GLenum;
typedef unsigned int GLbitfield;
typedef unsigned int GLuint;
#define GL_FALSE 0
struct SDL_Surface { int _d; };
struct TinyImage { int _d; };
void   glEnableVertexAttribArray(unsigned){}
void   glDisableVertexAttribArray(unsigned){}
void   glVertexAttribFormat(unsigned,int,unsigned,unsigned,unsigned){}
void   glVertexBindingDivisor(unsigned,unsigned){}
void   glVertexAttribBinding(unsigned,unsigned){}
void   glBindVertexBuffer(unsigned,size_t,size_t,size_t){}
unsigned glGenVertexArrays(unsigned n,unsigned* o){if(o)*o=0;return 0;}
void   glBindVertexArray(unsigned){}
unsigned glGenBuffers(unsigned n,unsigned* o){if(o)*o=0;return 0;}
void   glBindBuffer(unsigned,unsigned){}
void   glBufferStorage(unsigned,unsigned,void*,unsigned){}
void   glBufferData(unsigned,size_t,const void*,unsigned){}
void*  glMapBufferRange(unsigned,unsigned,unsigned,unsigned){return nullptr;}
void   glUnmapBuffer(unsigned){}
void   glDeleteBuffers(unsigned n,const unsigned*){}
void   glDeleteVertexArrays(unsigned n,const unsigned*){}
void   glClearColor(float,float,float,float){}
void   glClear(unsigned){}
void   glPixelStorei(unsigned,unsigned){}
void   glTexImage2D(unsigned,int,int,unsigned,unsigned,unsigned,unsigned,unsigned,const void*){}
void   glGenerateMipmap(unsigned){}
void   glMultiDrawElementsIndirect(unsigned,unsigned,const void*,size_t,size_t){}

// ─── GLM ─────────────────────────────────────────────────────────────────
#include <glm/glm.hpp>

// ─── Simulation config ───────────────────────────────────────────────────
#define POOLSIZE 10000000
static int SIZEX = 256;
static int SIZEY = 256;
static int SCALE = 80;
static int SLICE = 2 * SCALE;
static int NWIND = 250;
static int NWATER = 250;
static uint32_t SEED = 0;

// ─── Simulation headers ──────────────────────────────────────────────────
#include "source/include/vertexpool.h"
#include "source/layermap.h"
#include "source/particle/water.h"
#include "source/particle/wind.h"

// io.h needs TinyEngine image:: make/save -- stub
namespace image {
  SDL_Surface* make(std::function<vec4(vec2)>, ivec2) { return nullptr; }
  void save(SDL_Surface* s, const char*) {}
  void save(SDL_Surface* s, const std::string& f) { save(s, f.c_str()); }
}

#include "source/io.h"

// ─── Simulation wrapper ──────────────────────────────────────────────────
struct Sim {
    uint32_t seed;
    bool initialized = false;
    Layermap terrain;
    int32_t dim_x;
    int32_t dim_y;
    int32_t nwater = 250;
    int32_t nwind = 250;
    int32_t scale = 80;
    Vertexpool<Vertex> vpool;


    Sim(uint32_t seed_, int dim_x, int dim_y, const char* soil_file)
        : dim_x(dim_x), dim_y(dim_y), nwater(250), nwind(250), seed(seed_), terrain((int)seed_, ivec2(dim_x, dim_y)), vpool(dim_x * dim_y * 4, 256)
    {
        soils.clear();
        soilmap.clear();
        layers.clear();
        phong.clear();
        soils.emplace_back(SurfParam{ "Air", 0.0f, 1.0f, vec4(0.0f, 0.2f, 0.4f, 1.0f), vec4(0.5f, 0.8f, 0.2f, 32.0f), 0, 0.0f, 0.0f, 0.0f, 0, 0.0f, 0, 0.0f, 0.0f, 0, 0.0f, 0.0f });
        soilmap["Air"] = 0;
        SIZEX = dim_x;
        SIZEY = dim_y;
        SCALE = 80;
        SLICE = 2 * SCALE;
        srand(seed_);
        if (soil_file && soil_file[0])
            loadsoil(soil_file);
        else
            loadsoil("soil/default.soil");
        WaterParticle::init();
        WindParticle::init();
        initialized = true;
    }
    void reset() {
        // Delete allocated arrays before clearing pointers
        if (WaterParticle::frequency) { delete[] WaterParticle::frequency; WaterParticle::frequency = nullptr; }
        if (WaterParticle::track) { delete[] WaterParticle::track; WaterParticle::track = nullptr; }
        if (WindParticle::frequency) { delete[] WindParticle::frequency; WindParticle::frequency = nullptr; }
        terrain.~Layermap();
        vpool.clear();
        soils.clear();
        soilmap.clear();
        layers.clear();
        phong.clear();
    }
    ~Sim() {
        if (WaterParticle::frequency) { delete[] WaterParticle::frequency; }
        if (WaterParticle::track) { delete[] WaterParticle::track; }
        if (WindParticle::frequency) { delete[] WindParticle::frequency; }
        terrain.~Layermap();
        vpool.clear();
        soils.clear();
        soilmap.clear();
        layers.clear();
        phong.clear();
    }

    void step_water(int n) {
        if (!initialized) return;
        for (int i = 0; i < n; i++) {
            WaterParticle particle(terrain);
            while (true) {
                while (particle.move(terrain, vpool) && particle.interact(terrain, vpool));
                if (!particle.flood(terrain, vpool))
                    break;
            }
        }
    }

    void step_wind(int n) {
        if (!initialized) return;
        for (int i = 0; i < n; i++) {
            WindParticle particle(terrain);
            while (particle.move(terrain, vpool) && particle.interact(terrain, vpool));
        }
    }

    double height(int x, int y) {
        if (!initialized) return 0.0;
        return terrain.height(ivec2(x, y));
    }

    void heightmap(float* out) {
        if (!initialized) return;
        int n = SIZEX * SIZEY;
        for (int i = 0; i < n; i++) {
            out[i] = (float)terrain.height(ivec2(i % SIZEX, i / SIZEX));
        }
    }

    void colormap(uint8_t* out, int* len) {
        if (!initialized) return;
        int n = SIZEX * SIZEY;
        for (int i = 0; i < n; i++) {
            ivec2 p(i % SIZEX, i / SIZEX);
            sec* top = terrain.top(p);
            if (top != nullptr) {
                SurfParam& param = soils[top->type];
                out[i*4+0] = (uint8_t)(param.color.x * 255.0f);
                out[i*4+1] = (uint8_t)(param.color.y * 255.0f);
                out[i*4+2] = (uint8_t)(param.color.z * 255.0f);
                out[i*4+3] = 255;
            } else {
                out[i*4+0] = 0; out[i*4+1] = 51;
                out[i*4+2] = 102; out[i*4+3] = 255;
            }
        }
        *len = n * 4;
    }

    void normal(float* nx, float* ny, float* nz, int x, int y) {
        if (!initialized) { *nx = *ny = 0.0f; *nz = 1.0f; return; }
        vec3 n = terrain.normal(ivec2(x, y));
        *nx = n.x; *ny = n.y; *nz = n.z;
    }

    void set_seed(uint32_t s) { seed = s; }
    void set_nwater(int n) { nwater = n; }
    void set_nwind(int n) { nwind = n; }
    void set_scale(int s) { SCALE = s; }
    int32_t get_dim_x() const { return dim_x; }
    int32_t get_dim_y() const { return dim_y; }
};

// ─── FFI exports ─────────────────────────────────────────────────────────
extern "C" {
void* sim_create(uint32_t seed, int dim_x, int dim_y, const char* soil_file) { return new Sim(seed, dim_x, dim_y, soil_file); }
void  sim_destroy(void* raw) { delete (Sim*)raw; }
void  sim_step_water(void* raw, int n) { ((Sim*)raw)->step_water(n); }
void  sim_step_wind(void* raw, int n) { ((Sim*)raw)->step_wind(n); }
double sim_get_height(void* raw, int x, int y) { return ((Sim*)raw)->height(x, y); }
void  sim_get_heightmap(void* raw, float* out) { ((Sim*)raw)->heightmap(out); }
void  sim_get_color_map(void* raw, uint8_t* out, int* len) { ((Sim*)raw)->colormap(out, len); }
void  sim_get_normal_x(void* raw, int x, int y, float* out) { float ny=0,nz=0; ((Sim*)raw)->normal(out,&ny,&nz,x,y); }
void  sim_get_normal_y(void* raw, int x, int y, float* out) { float nx=0,nz=0; ((Sim*)raw)->normal(&nx,out,&nz,x,y); }
void  sim_get_normal_z(void* raw, int x, int y, float* out) { float nx=0,ny=0; ((Sim*)raw)->normal(&nx,&ny,out,x,y); }
void  sim_set_seed(void* raw, uint32_t seed) { ((Sim*)raw)->set_seed(seed); }
void  sim_set_nwater(void* raw, int n) { ((Sim*)raw)->set_nwater(n); }
void  sim_set_nwind(void* raw, int n) { ((Sim*)raw)->set_nwind(n); }
void  sim_set_scale(void* raw, int s) { ((Sim*)raw)->set_scale(s); }
int   sim_get_dim_x(void* raw) { return ((Sim*)raw)->get_dim_x(); }
int   sim_get_dim_y(void* raw) { return ((Sim*)raw)->get_dim_y(); }
void  sim_reset(void* raw) { ((Sim*)raw)->reset(); }

}
