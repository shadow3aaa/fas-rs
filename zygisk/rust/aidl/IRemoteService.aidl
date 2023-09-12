interface IRemoteService {
    /** Send pkg name & frametime to fas-rs server. */
    boolean sendFrameData(String Pkg, long FrameTimeNanos);
}