interface IRemoteService {
    boolean sendData(long buffer, String Pkg, int pid, long FrameTimeNanos, int cpu);
    void removeBuffer(long buffer, int pid);
}