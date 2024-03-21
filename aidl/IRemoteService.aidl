interface IRemoteService {
    boolean needFas(String pkg);
    boolean sendData(int pid, long FrameTimeNanos);
}
