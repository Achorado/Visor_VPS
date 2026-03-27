<script lang="ts">
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';

  interface Props {
    data?: number[];
    color?: string;
    width?: string;
    height?: string;
  }

  let { data = [], color = '#6366f1', width = '80px', height = '32px' }: Props = $props();

  let container: HTMLDivElement;
  let chart: echarts.ECharts | null = null;

  function buildOption(d: number[]) {
    const now = Date.now();
    const series = d.map((v, i) => [now - (d.length - 1 - i) * 5000, v]);
    return {
      animation: false,
      grid: { top: 2, right: 2, bottom: 2, left: 2 },
      xAxis: { type: 'time', show: false },
      yAxis: { type: 'value', show: false, min: 0, max: 100 },
      series: [{
        type: 'line',
        data: series,
        smooth: 0.5,
        symbol: 'none',
        lineStyle: { color, width: 1.5 },
        areaStyle: {
          color: {
            type: 'linear', x: 0, y: 0, x2: 0, y2: 1,
            colorStops: [{ offset: 0, color: `${color}66` }, { offset: 1, color: `${color}00` }]
          }
        }
      }]
    };
  }

  onMount(() => {
    chart = echarts.init(container, 'dark');
    chart.setOption(buildOption(data));
    return () => chart?.dispose();
  });

  $effect(() => {
    chart?.setOption(buildOption(data), { notMerge: true });
  });
</script>

<div bind:this={container} style:width style:height></div>
