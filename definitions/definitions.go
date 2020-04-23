package definitions

// AudioInterface an interface for audio interfaces. This
// service runs on bare metal and is capable of interacting
// with the hardware. Its purpose is to open streams and
// forward the raw audio to a UDP port.
type AudioInterface interface {
	CreateStream(CreateStreamRequest) CreateStreamResponse

	DeleteStream(DeleteStreamRequest) DeleteStreamResponse

	ListStreams(ListStreamsRequest) ListStreamsResponse

	GetDeviceInfo(GetDeviceInfoRequest) GetDeviceInfoResponse
}

// HardwarePort defines a single IO port on an audio interface.
// Notably, this struct does not contain informataion about it
// being an input or output. SampleRate typically must match the
// device driver. If there is disagreement, an error is returned.
type HardwarePort struct {
	Channel      int    `json:"channel"`      // Correspondance with physical input
	SampleRate   int    `json:"sampleRate"`   // e.g. 192000. Must match driver.
	SampleFormat string `json:"sampleFormat"` // I16, U16, or F32
}

// Address defines a UDP address for sending or receiving raw audio.
type Address struct {
	Host string `json:"host"`
	Port int    `json:"port"`
}

// Stream defines a unidirectional flow of audio involving a hardware
// input (probably XLR) and a remote port. Default is input behavior,
// where the input of the interface the audio to the remote port.
// The opposite (IsOutput == true) reverses the direction. The host
// address used for outputs should be 0.0.0.0 (all network interfaces).
type Stream struct {
	IsOutput     bool         `json:"isOutput"`     // If true, direction is Address -> HardwarePort
	Address      Address      `json:"address"`      //
	HardwarePort HardwarePort `json:"hardwarePort"` //
}

// StreamWithMetrics boxes a Stream object with some metrics
// collected over its lifetime. TotalSent and TotalReceive
// are intentionally disambiguated, even though one will
// always be zero (streams are unidirectional).
type StreamWithMetrics struct {
	Stream        Stream `json:"stream"`        //
	TotalSent     int    `json:"totalSent"`     // Total bytes sent since creation (inputs only)
	TotalReceived int    `json:"totalReceived"` // Total bytes received since creation (outputs only)
	Created       string `json:"created"`       // time.UnixDate format
}

// CreateStreamRequest ...
type CreateStreamRequest struct {
	Stream Stream `json:"stream"`
}

// CreateStreamResponse ...
type CreateStreamResponse struct {
	Created string `json:"created"` // time.UnixDate format
}

// DeleteStreamRequest request to delete a stream
type DeleteStreamRequest struct {
	Stream Stream `json:"stream"`
}

// DeleteStreamResponse ...
type DeleteStreamResponse struct {
}

// ListStreamsRequest ...
type ListStreamsRequest struct {
}

// ListStreamsResponse aggregates all existing streams with
// relevant metrics.
type ListStreamsResponse struct {
	Streams []StreamWithMetrics `json:"streams"`
}

// ChannelInfo enumerates valid configuration for the channel.
// Any of these values passed to CreateStream should not yield
// an unsupported config error.
type ChannelInfo struct {
	SampleRates []int  `json:"sampleRates"`    // e.g. [48000, 96000, 192000] but typically only one value is present
	Desc        string `json:"desc,omitempty"` // Optional description, e.g. "front left"
}

// DeviceInfo enumerates valid configs for the device
type DeviceInfo struct {
	Channels []ChannelInfo `json:"channels"`
}

// GetDeviceInfoRequest ...
type GetDeviceInfoRequest struct {
}

// GetDeviceInfoResponse ...
type GetDeviceInfoResponse struct {
	Info DeviceInfo `json:"info"`
}
