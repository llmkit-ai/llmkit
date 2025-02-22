<template>
  <div class="chart-container">
    <canvas ref="chartRef"></canvas>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import Chart from "chart.js/auto";
import type { PromptEvalVersionPerformanceResponse } from "~/types/response/prompts";

const props = defineProps<{
  performance: PromptEvalVersionPerformanceResponse[];
  promptName: string;
}>();

const chartRef = ref<HTMLCanvasElement | null>(null);

onMounted(() => {
  if (!chartRef.value) return;

  new Chart(chartRef.value, {
    type: "line",
    data: {
      labels: props.performance.map((p) => `v${p.version_number}`),
      datasets: [
        {
          label: "Average Score",
          data: props.performance.map((p) => p.avg_score ?? 0),
          borderColor: "rgb(75, 192, 192)",
          backgroundColor: "rgba(75, 192, 192, 0.1)",
          tension: 0.1,
          fill: true,
        },
      ],
    },
    options: {
      responsive: true,
      plugins: {
        title: {
          display: true,
          text: `${props.promptName} Performance History`,
        },
        tooltip: {
          callbacks: {
            label: (context) => {
              const p = props.performance[context.dataIndex];
              return [
                `Score: ${p.avg_score?.toFixed(2) ?? "N/A"}`,
                `Runs: ${p.run_count}`,
                `Date: ${new Date(p.version_date).toLocaleDateString()}`,
              ];
            },
          },
        },
      },
      scales: {
        y: {
          beginAtZero: true,
          max: 5,
          title: {
            display: true,
            text: "Average Score",
          },
        },
        x: {
          title: {
            display: true,
            text: "Version",
          },
        },
      },
    },
  });
});
</script>

<style scoped>
.chart-container {
  width: 100%;
  max-width: 900px;
  margin: 0 auto;
}
</style>
