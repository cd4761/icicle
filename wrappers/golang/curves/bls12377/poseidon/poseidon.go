package poseidon

// #cgo CFLAGS: -I./include/
// #include "poseidon.h"
import "C"
import (
	"runtime"
	"unsafe"

	"github.com/ingonyama-zk/icicle/v2/wrappers/golang/core"
	cr "github.com/ingonyama-zk/icicle/v2/wrappers/golang/cuda_runtime"
	bls12_377 "github.com/ingonyama-zk/icicle/v2/wrappers/golang/curves/bls12377"
)

type PoseidonHandler = C.struct_PoseidonInst
type Poseidon struct {
	width  uint32
	handle *PoseidonHandler
}

func Create(arity uint32, alpha uint32, fullRoundsHalf uint32, partialRounds uint32, scalars core.HostOrDeviceSlice, mdsMatrix core.HostOrDeviceSlice, nonSparseMatrix core.HostOrDeviceSlice, sparseMatrices core.HostOrDeviceSlice, domainTag bls12_377.ScalarField, ctx *cr.DeviceContext) (*Poseidon, core.IcicleError) {
	var poseidon *PoseidonHandler
	cArity := (C.uint)(arity)
	cAlpha := (C.uint)(alpha)
	cFullRoundsHalf := (C.uint)(fullRoundsHalf)
	cPartialRounds := (C.uint)(partialRounds)
	cScalars := (*C.scalar_t)(scalars.AsUnsafePointer())
	cMdsMatrix := (*C.scalar_t)(mdsMatrix.AsUnsafePointer())
	cNonSparseMatrix := (*C.scalar_t)(nonSparseMatrix.AsUnsafePointer())
	cSparseMatrices := (*C.scalar_t)(sparseMatrices.AsUnsafePointer())
	cDomainTag := (*C.scalar_t)(unsafe.Pointer(&domainTag))
	cCtx := (*C.DeviceContext)(unsafe.Pointer(ctx))
	__ret := C.bls12_377_poseidon_create_cuda(&poseidon, cArity, cAlpha, cFullRoundsHalf, cPartialRounds, cScalars, cMdsMatrix, cNonSparseMatrix, cSparseMatrices, cDomainTag, cCtx)
	err := core.FromCudaError((cr.CudaError)(__ret))
	if err.IcicleErrorCode != core.IcicleSuccess {
		return nil, err
	}
	p := Poseidon{handle: poseidon, width: arity + 1}
	runtime.SetFinalizer(&p, func(p *Poseidon) {
		p.Delete()
	})
	return &p, err
}

func Load(arity uint32, ctx *cr.DeviceContext) (*Poseidon, core.IcicleError) {
	var poseidon *PoseidonHandler
	cArity := (C.uint)(arity)
	cCtx := (*C.DeviceContext)(unsafe.Pointer(ctx))
	__ret := C.bls12_377_poseidon_load_cuda(&poseidon, cArity, cCtx)
	err := core.FromCudaError((cr.CudaError)(__ret))
	if err.IcicleErrorCode != core.IcicleSuccess {
		return nil, err
	}
	p := Poseidon{handle: poseidon, width: arity + 1}
	runtime.SetFinalizer(&p, func(p *Poseidon) {
		p.Delete()
	})
	return &p, err
}

func (poseidon *Poseidon) HashMany(inputs core.HostOrDeviceSlice, output core.HostOrDeviceSlice, numberOfStates uint32, inputBlockLen uint32, outputLen uint32, cfg *core.HashConfig) core.IcicleError {
	core.SpongeInputCheck(inputs, numberOfStates, inputBlockLen, cfg.InputRate, &cfg.Ctx)
	core.SpongeOutputsCheck(output, numberOfStates, outputLen, poseidon.width, false, &cfg.Ctx)

	cInputs := (*C.scalar_t)(inputs.AsUnsafePointer())
	cOutput := (*C.scalar_t)(output.AsUnsafePointer())
	cNumberOfStates := (C.uint)(numberOfStates)
	cInputBlockLen := (C.uint)(inputBlockLen)
	cOutputLen := (C.uint)(outputLen)
	cCfg := (*C.HashConfig)(unsafe.Pointer(cfg))
	__ret := C.bls12_377_poseidon_hash_many_cuda(poseidon.handle, cInputs, cOutput, cNumberOfStates, cInputBlockLen, cOutputLen, cCfg)
	err := (cr.CudaError)(__ret)
	return core.FromCudaError(err)
}

func (poseidon *Poseidon) Delete() core.IcicleError {
	__ret := C.bls12_377_poseidon_delete_cuda(poseidon.handle)
	err := (cr.CudaError)(__ret)
	return core.FromCudaError(err)
}

func (poseidon *Poseidon) GetDefaultHashConfig() core.HashConfig {
	cfg := core.GetDefaultHashConfig()
	cfg.InputRate = poseidon.width - 1
	cfg.OutputRate = poseidon.width
	return cfg
}
