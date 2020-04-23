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

// Source defines a single input channel on an audio interface.
// Generally, SampleRate must match the device driver. If there
// is disagreement, an error is returned.
type Source struct {
	Channel      int    `json:"channel"`      // Unambiguous correspondance with physical input
	SampleRate   int    `json:"sampleRate"`   // e.g. 192000. Must match driver.
	SampleFormat string `json:"sampleFormat"` // I16, U16, or F32
}

// Sink defines a UDP destination for sending the audio stream
type Sink struct {
	Host string `json:"host"`
	Port int    `json:"port"`
}

// Stream describes a stream of audio from a hardware input
// to a destination IP address. The sink uniquely identifies
// the stream, so an error should be returned if there is
// an attempt to create multiple streams with the same sink.
type Stream struct {
	Source Source `json:"source"` // Hardware source
	Sink   Sink   `json:"sink"`   // Network sink
}

// StreamWithMetrics boxes a Stream object with some metrics
// collected over its lifetime.
type StreamWithMetrics struct {
	Stream    Stream `json:"stream"`    //
	TotalSent int    `json:"totalSent"` // Total bytes sent since creation
	Started   string `json:"started"`   // time.UnixDate format
	LocalPort int    `json:"localPort"` // Local outbound UDP port
}

// CreateStreamRequest ...
type CreateStreamRequest struct {
	Stream Stream `json:"stream"`
}

// CreateStreamResponse ...
type CreateStreamResponse struct {
}

// DeleteStreamRequest ...
type DeleteStreamRequest struct {
	Sink Sink `json:"sink"` // Uniquely identifies the stream
}

// DeleteStreamResponse ...
type DeleteStreamResponse struct {
}

// ListStreamsRequest ...
type ListStreamsRequest struct {
	FilterChannel *int `json:"filterChannel,omitempty"`
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
