pub enum TcpConnectionStatus {
	Established,
	SynSent,
	SynRecv,
	FinWait1,
	FinWait2,
	TimeWait,
	Close,
	CloseWait,
	LastAck,
	Listen,
	Closing,
	None,
	/// Windows only
	DeleteTcb,
	/// Solaris only
	Idle,
	/// Solaris only
	Bound,
}
