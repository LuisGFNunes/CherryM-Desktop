import { useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { 
  Home, 
  Search, 
  Library, 
  Play, 
  Pause,
  SkipBack, 
  SkipForward, 
  Volume2, 
  Cherry,
  Loader2
} from "lucide-react";
import "./App.css";

interface Song {
  id: string;
  title: string;
  artist: string;
  thumbnail: string;
  duration: string;
}

export default function App() {
  const [searchQuery, setSearchQuery] = useState("");
  const [songs, setSongs] = useState<Song[]>([]);
  const [loadingSearch, setLoadingSearch] = useState(false);
  
  const [currentTrack, setCurrentTrack] = useState<Song | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [loadingAudio] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [volume, setVolume] = useState(0.8);

  const audioRef = useRef<HTMLAudioElement | null>(null);

  const handleSearch = async (e: React.SubmitEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!searchQuery.trim()) return;

    setLoadingSearch(true);
    try {
      const results = await invoke<Song[]>("search_songs", { query: searchQuery });
      setSongs(results);
    } catch (err) {
      console.error("Erro na busca:", err);
    } finally {
      setLoadingSearch(false);
    }
  };

  const playSong = (song: Song) => {
    setCurrentTrack(song);
    setIsPlaying(false);

    if (audioRef.current) {
      const streamUrl = `http://127.0.0.1:9876/stream/${song.id}`;
    
      audioRef.current.src = streamUrl;
      audioRef.current.volume = volume;

      audioRef.current.load();

      audioRef.current
        .play()
        .then(() => {
          setIsPlaying(true);
        })
        .catch((err) => {
          console.error("Erro ao dar play no áudio:", err);
        });
    }
  };

  const togglePlay = () => {
    if (!audioRef.current || !currentTrack) return;
    if (isPlaying) {
      audioRef.current.pause();
      setIsPlaying(false);
    } else {
      audioRef.current.play();
      setIsPlaying(true);
    }
  };

  const formatTime = (time: number) => {
    if (isNaN(time)) return "0:00";
    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60);
    return `${minutes}:${seconds < 10 ? "0" : ""}${seconds}`;
  };

  const handleVolumeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = parseFloat(e.target.value);
    setVolume(val);
    if (audioRef.current) {
      audioRef.current.volume = val;
    }
  };

  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    const targetTime = parseFloat(e.target.value);
    setCurrentTime(targetTime);
    if (audioRef.current) {
      audioRef.current.currentTime = targetTime;
    }
  };

  return (
    <div className="app-container">
      {/* Hidden Native Player */}
      <audio
        ref={audioRef}
        onTimeUpdate={() => audioRef.current && setCurrentTime(audioRef.current.currentTime)}
        onLoadedMetadata={() => audioRef.current && setDuration(audioRef.current.duration)}
        onEnded={() => setIsPlaying(false)}
        onError={(e) => {
          console.error("Erro na reprodução do áudio:", e);
          setIsPlaying(false);
        }}
      />

      {/* Sidebar */}
      <aside className="sidebar glass-panel">
        <div className="brand">
          <Cherry size={28} color="#FFB7C5" />
          <span>Cherry</span>
        </div>

        <nav className="nav-menu">
          <div className="nav-item active">
            <Home size={20} />
            <span>Início</span>
          </div>
          <div className="nav-item">
            <Search size={20} />
            <span>Buscar</span>
          </div>
          <div className="nav-item">
            <Library size={20} />
            <span>Sua Biblioteca</span>
          </div>
        </nav>
      </aside>

      {/* Main Content */}
      <main className="main-content glass-panel">
        <form onSubmit={handleSearch} className="search-bar">
          <Search size={18} color="rgba(255, 255, 255, 0.6)" />
          <input 
            type="text" 
            placeholder="Pesquisar músicas, artistas ou álbuns..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
          {loadingSearch && <Loader2 size={18} className="spin" color="#FFB7C5" />}
        </form>

        <div className="results-container">
          <h2 className="section-title">
            {songs.length > 0 ? "Resultados da Busca" : "Descubra novas músicas"}
          </h2>

          <div className="songs-grid">
            {songs.map((song) => (
              <div 
                key={song.id} 
                className="song-card"
                onClick={() => playSong(song)}
              >
                <div className="card-cover-wrapper">
                  <img src={song.thumbnail} alt={song.title} className="card-cover" />
                  <button className="card-play-btn">
                    <Play size={20} fill="#0B0E17" color="#0B0E17" />
                  </button>
                </div>
                <div className="song-title">{song.title}</div>
                <div className="song-artist">{song.artist}</div>
              </div>
            ))}
          </div>
        </div>
      </main>

      {/* Player Footer */}
      <footer className="player-bar glass-panel">
        <div className="track-info">
          {currentTrack ? (
            <>
              <img src={currentTrack.thumbnail} alt="Cover" className="track-cover" />
              <div style={{ overflow: "hidden" }}>
                <div className="current-title">{currentTrack.title}</div>
                <div className="current-artist">{currentTrack.artist}</div>
              </div>
            </>
          ) : (
            <>
              <div className="track-cover-placeholder" />
              <div>
                <div className="current-title">Nenhuma música</div>
                <div className="current-artist">Selecione uma faixa</div>
              </div>
            </>
          )}
        </div>

        <div className="player-controls">
          <div className="control-buttons">
            <SkipBack size={18} color="rgba(255, 255, 255, 0.6)" cursor="pointer" />
            
            <button className="play-btn" onClick={togglePlay} disabled={loadingAudio || !currentTrack}>
              {loadingAudio ? (
                <Loader2 size={18} className="spin" color="#0B0E17" />
              ) : isPlaying ? (
                <Pause size={18} fill="#0B0E17" color="#0B0E17" />
              ) : (
                <Play size={18} fill="#0B0E17" color="#0B0E17" />
              )}
            </button>

            <SkipForward size={18} color="rgba(255, 255, 255, 0.6)" cursor="pointer" />
          </div>

          <div className="progress-bar-container">
            <span>{formatTime(currentTime)}</span>
            <input 
              type="range" 
              min={0} 
              max={duration || 0} 
              value={currentTime} 
              onChange={handleSeek}
              className="progress-slider"
            />
            <span>{formatTime(duration)}</span>
          </div>
        </div>

        <div className="volume-control">
          <Volume2 size={18} color="rgba(255, 255, 255, 0.6)" />
          <input 
            type="range" 
            min={0} 
            max={1} 
            step={0.01} 
            value={volume} 
            onChange={handleVolumeChange}
            className="volume-slider"
          />
        </div>
      </footer>
    </div>
  );
}