package keeper

import (
	"github.com/enigmampc/EnigmaBlockchain/x/registration/internal/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"io/ioutil"
	"os"
	"testing"
)

func TestInitGenesisNoMaster(t *testing.T) {
	tempDir, err := ioutil.TempDir("", "wasm")
	require.NoError(t, err)
	defer os.RemoveAll(tempDir)
	ctx, keeper := CreateTestInput(t, false, tempDir, true)
	//
	//cert, err := ioutil.ReadFile("../../testdata/attestation_cert")
	//require.NoError(t, err)

	data := types.GenesisState{
		Registration:              nil,
		IoMasterCertificate:       nil,
		NodeExchMasterCertificate: nil,
	}

	assert.Panics(t, func() { InitGenesis(ctx, keeper, data) }, "Init genesis didn't panic without master certificate")
}

func TestInitGenesis(t *testing.T) {
	tempDir, err := ioutil.TempDir("", "wasm")
	require.NoError(t, err)
	defer os.RemoveAll(tempDir)
	ctx, keeper := CreateTestInput(t, false, tempDir, true)

	cert, err := ioutil.ReadFile("../../testdata/attestation_cert")
	require.NoError(t, err)

	data := types.GenesisState{
		Registration:              nil,
		IoMasterCertificate:       cert,
		NodeExchMasterCertificate: cert,
	}

	InitGenesis(ctx, keeper, data)
}

func TestExportGenesis(t *testing.T) {
	tempDir, err := ioutil.TempDir("", "wasm")
	require.NoError(t, err)
	defer os.RemoveAll(tempDir)
	ctx, keeper := CreateTestInput(t, false, tempDir, true)

	cert, err := ioutil.ReadFile("../../testdata/attestation_cert")
	require.NoError(t, err)

	data := types.GenesisState{
		Registration:              nil,
		IoMasterCertificate:       cert,
		NodeExchMasterCertificate: cert,
	}

	InitGenesis(ctx, keeper, data)

	data2 := ExportGenesis(ctx, keeper)

	require.Equal(t, string(data.IoMasterCertificate), string(data2.IoMasterCertificate))
	require.Equal(t, string(data.NodeExchMasterCertificate), string(data2.NodeExchMasterCertificate))
	require.Equal(t, data2.Registration, data2.Registration)
}
