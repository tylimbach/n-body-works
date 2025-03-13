#include <cuda_runtime.h>
#include <stdio.h>
#include <math.h>

#ifdef _WIN32
#define API_EXPORT __declspec(dllexport)
#else
#define API_EXPORT
#endif

__global__ void compute_accelerations_kernel(const float3* positions,
                                               const float* masses,
                                               float3* accelerations,
                                               int particle_count,
                                               float g,
                                               float softening)
{
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < particle_count) {
        float3 pos_i = positions[i];
        float3 acc = make_float3(0.0f, 0.0f, 0.0f);
        for (int j = 0; j < particle_count; j++) {
            if (i == j) continue;
            float3 pos_j = positions[j];
            float dx = pos_j.x - pos_i.x;
            float dy = pos_j.y - pos_i.y;
            float dz = pos_j.z - pos_i.z;
            float dist_sqr = dx * dx + dy * dy + dz * dz + softening;
            // Use rsqrtf for fast reciprocal square-root
            float inv_dist = rsqrtf(dist_sqr);
            float inv_dist3 = inv_dist * inv_dist * inv_dist;
            float force = g * masses[j] * inv_dist3;
            acc.x += force * dx;
            acc.y += force * dy;
            acc.z += force * dz;
        }
        accelerations[i] = acc;
    }
}

extern "C" API_EXPORT void compute_accelerations(const float* host_positions,
                                      const float* host_masses,
                                      float* host_accelerations,
                                      int particle_count,
                                      float g,
                                      float softening)
{
    float3 *d_positions = nullptr;
    float  *d_masses = nullptr;
    float3 *d_accelerations = nullptr;

    size_t numParticles = particle_count;
    size_t positions_size = numParticles * sizeof(float3);
    size_t masses_size = numParticles * sizeof(float);
    size_t accelerations_size = numParticles * sizeof(float3);

    cudaMalloc((void**)&d_positions, positions_size);
    cudaMalloc((void**)&d_masses, masses_size);
    cudaMalloc((void**)&d_accelerations, accelerations_size);

    cudaMemcpy(d_positions, host_positions, positions_size, cudaMemcpyHostToDevice);
    cudaMemcpy(d_masses, host_masses, masses_size, cudaMemcpyHostToDevice);

    int threadsPerBlock = 256;
    int blocks = (particle_count + threadsPerBlock - 1) / threadsPerBlock;
    compute_accelerations_kernel<<<blocks, threadsPerBlock>>>(d_positions, d_masses, d_accelerations,
                                                              particle_count, g, softening);
    cudaDeviceSynchronize();

    cudaMemcpy(host_accelerations, d_accelerations, accelerations_size, cudaMemcpyDeviceToHost);

    cudaFree(d_positions);
    cudaFree(d_masses);
    cudaFree(d_accelerations);
}
