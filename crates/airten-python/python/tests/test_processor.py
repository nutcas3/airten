import numpy as np
import pytest

import airten


class TestAudioProcessor:
    def test_create_default(self):
        proc = airten.AudioProcessor()
        assert proc.sample_rate == 48000
        assert proc.frame_size == 512

    def test_create_custom(self):
        proc = airten.AudioProcessor(
            sample_rate=44100,
            frame_size=256,
            channels=2,
            noise_suppression=False,
            compression=False,
        )
        assert proc.sample_rate == 44100
        assert proc.frame_size == 256

    def test_process(self):
        proc = airten.AudioProcessor()
        samples = np.random.randn(512).astype(np.float32) * 0.5
        
        output = proc.process(samples)
        
        assert output.shape == samples.shape
        assert output.dtype == np.float32

    def test_process_inplace(self):
        proc = airten.AudioProcessor()
        samples = np.random.randn(512).astype(np.float32) * 0.5
        original = samples.copy()
        
        proc.process_inplace(samples)
        
        # Samples should be modified
        assert not np.allclose(samples, original)

    def test_reset(self):
        proc = airten.AudioProcessor()
        samples = np.random.randn(512).astype(np.float32) * 0.5
        
        proc.process(samples)
        proc.reset()
        
        # Should not raise
        proc.process(samples)

    def test_envelope_level(self):
        proc = airten.AudioProcessor()
        samples = np.ones(512, dtype=np.float32) * 0.5
        
        proc.process(samples)
        
        assert proc.envelope_level >= 0


class TestFilter:
    def test_lowpass(self):
        filt = airten.Filter.lowpass(48000, 1000, 0.707)
        samples = np.random.randn(256).astype(np.float32)
        
        output = filt.process(samples)
        
        assert output.shape == samples.shape

    def test_highpass(self):
        filt = airten.Filter.highpass(48000, 100, 0.707)
        samples = np.random.randn(256).astype(np.float32)
        
        output = filt.process(samples)
        
        assert output.shape == samples.shape

    def test_bandpass(self):
        filt = airten.Filter.bandpass(48000, 1000, 2.0)
        samples = np.random.randn(256).astype(np.float32)
        
        output = filt.process(samples)
        
        assert output.shape == samples.shape

    def test_peaking(self):
        filt = airten.Filter.peaking(48000, 1000, 1.0, 6.0)
        samples = np.random.randn(256).astype(np.float32)
        
        output = filt.process(samples)
        
        assert output.shape == samples.shape


class TestCompressor:
    def test_create(self):
        comp = airten.DynamicCompressor(48000)
        assert comp is not None

    def test_settings(self):
        comp = airten.DynamicCompressor(48000)
        
        comp.set_threshold(-20.0)
        comp.set_ratio(4.0)
        comp.set_attack(10.0)
        comp.set_release(100.0)
        comp.set_knee(6.0)
        comp.set_makeup_gain(3.0)

    def test_process(self):
        comp = airten.DynamicCompressor(48000)
        samples = np.random.randn(512).astype(np.float32)
        
        comp.process_inplace(samples)
        
        assert samples.dtype == np.float32


class TestGate:
    def test_create(self):
        gate = airten.Gate(48000)
        assert gate is not None

    def test_settings(self):
        gate = airten.Gate(48000)
        
        gate.set_threshold(-40.0)
        gate.set_attack(1.0)
        gate.set_hold(50.0)
        gate.set_release(100.0)
        gate.set_range(-80.0)

    def test_is_open(self):
        gate = airten.Gate(48000)
        
        # Initially closed
        assert not gate.is_open


class TestResampler:
    def test_downsample(self):
        resampler = airten.Resampler(48000, 16000)
        samples = np.random.randn(480).astype(np.float32)
        
        output = resampler.process(samples)
        
        # Should be roughly 1/3 the size
        assert len(output) < len(samples)

    def test_upsample(self):
        resampler = airten.Resampler(16000, 48000)
        samples = np.random.randn(160).astype(np.float32)
        
        output = resampler.process(samples)
        
        # Should be roughly 3x the size
        assert len(output) > len(samples)

    def test_ratio(self):
        resampler = airten.Resampler(48000, 16000)
        assert resampler.ratio == pytest.approx(3.0, rel=0.01)


class TestUtilityFunctions:

    def test_calculate_rms(self):
        samples = np.ones(100, dtype=np.float32)
        rms = airten.calculate_rms(samples)
        assert rms == pytest.approx(1.0, rel=0.001)

    def test_find_peak(self):
        samples = np.array([0.5, -0.8, 0.3], dtype=np.float32)
        peak = airten.find_peak(samples)
        assert peak == pytest.approx(0.8, rel=0.001)

    def test_db_to_linear(self):
        assert airten.db_to_linear(0) == pytest.approx(1.0, rel=0.001)
        assert airten.db_to_linear(-6) == pytest.approx(0.501, rel=0.01)
        assert airten.db_to_linear(6) == pytest.approx(1.995, rel=0.01)

    def test_linear_to_db(self):
        assert airten.linear_to_db(1.0) == pytest.approx(0.0, rel=0.001)
        assert airten.linear_to_db(0.5) == pytest.approx(-6.02, rel=0.1)
