interface IRemoteService {
    boolean sendData(long buffer, String Pkg, int pid, long FrameTimeNanos);
    void removeBuffer(long buffer, int pid);
}