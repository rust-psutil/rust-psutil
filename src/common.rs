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
	/// Windows only
	DeleteTcb,
	/// Solaris only
	Idle,
	/// Solaris only
	Bound,
}

pub enum NetConnectionType {
	Inet,
	Inet4,
	Inet6,
	Tcp,
	Tcp4,
	Tcp6,
	Udp,
	Udp4,
	Udp6,
	Unix,
	All,
}
