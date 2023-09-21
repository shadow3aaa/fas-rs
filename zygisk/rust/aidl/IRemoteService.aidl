interface IRemoteService {
    /** Send data to fas-rs server. */
    boolean sendData(String Pkg, int pid,  long FrameTimeNanos);
}