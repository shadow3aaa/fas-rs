interface IRemoteService {
    boolean needFas(String pkg);
    boolean sendData(long buffer, int pid, long FrameTimeNanos);
    void removeBuffer(long buffer, int pid);
}
