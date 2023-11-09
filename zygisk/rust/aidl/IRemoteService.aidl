interface IRemoteService {
    /** Send data to fas-rs server. */
    boolean sendData(long buffer, String Pkg, int pid, long FrameTimeNanos);
}